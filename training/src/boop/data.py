"""
Data loading utilities for Boop AlphaZero training.

Handles loading JSONL files produced by your Rust self-play,
converting to PyTorch tensors, and creating data loaders.
"""

import json
import numpy as np
import torch
from torch.utils.data import Dataset, DataLoader
from pathlib import Path
from typing import List, Tuple
import warnings


class BoopDataset(Dataset):
    """
    PyTorch Dataset for Boop training samples.
    
    Each sample contains:
    - state: (10, 6, 6) float array - board state
    - policy: (188,) float array - move probability distribution
    - value: float - game outcome from current player's perspective
    """
    
    def __init__(self, states, policies, values):
        """
        Args:
            states: numpy array of shape (N, 10, 6, 6)
            policies: numpy array of shape (N, 188)
            values: numpy array of shape (N,)
        """
        assert len(states) == len(policies) == len(values), "Mismatched lengths"
        
        self.states = torch.from_numpy(states).float()
        self.policies = torch.from_numpy(policies).float()
        self.values = torch.from_numpy(values).float()
        
    def __len__(self):
        return len(self.states)
    
    def __getitem__(self, idx):
        return self.states[idx], self.policies[idx], self.values[idx]


def load_jsonl(filepath: str) -> Tuple[np.ndarray, np.ndarray, np.ndarray]:
    """
    Load training data from a single JSONL file produced by Rust.
    
    Args:
        filepath: Path to .jsonl file
    
    Returns:
        states: (N, 10, 6, 6) numpy array
        policies: (N, 188) numpy array
        values: (N,) numpy array
    """
    states = []
    policies = []
    values = []
    
    filepath = Path(filepath)
    
    if not filepath.exists():
        raise FileNotFoundError(f"Data file not found: {filepath}")
    
    print(f"Loading data from {filepath}...")
    
    with open(filepath, 'r') as f:
        for line_num, line in enumerate(f, 1):
            line = line.strip()
            if not line:
                continue
            
            try:
                sample = json.loads(line)
                
                # Validate sample structure
                if not all(key in sample for key in ['state', 'policy', 'value']):
                    warnings.warn(f"Line {line_num}: Missing required keys, skipping")
                    continue
                
                # Extract and validate state
                state = np.array(sample['state'], dtype=np.float32)
                if state.shape != (360,):  # 10 * 6 * 6
                    warnings.warn(f"Line {line_num}: Invalid state shape {state.shape}, skipping")
                    continue
                state = state.reshape(10, 6, 6)
                
                # Extract and validate policy
                policy = np.array(sample['policy'], dtype=np.float32)
                if policy.shape != (188,):
                    warnings.warn(f"Line {line_num}: Invalid policy shape {policy.shape}, skipping")
                    continue
                
                # Normalize policy (should already be normalized, but just in case)
                policy_sum = policy.sum()
                if not np.isclose(policy_sum, 1.0, atol=1e-5):
                    if policy_sum > 0:
                        policy = policy / policy_sum
                    else:
                        warnings.warn(f"Line {line_num}: Policy sums to 0, skipping")
                        continue
                
                # Extract and validate value
                value = float(sample['value'])
                if not -1.0 <= value <= 1.0:
                    warnings.warn(f"Line {line_num}: Value {value} out of range [-1, 1], skipping")
                    continue
                
                states.append(state)
                policies.append(policy)
                values.append(value)
                
            except json.JSONDecodeError as e:
                warnings.warn(f"Line {line_num}: JSON decode error - {e}, skipping")
                continue
            except Exception as e:
                warnings.warn(f"Line {line_num}: Unexpected error - {e}, skipping")
                continue
    
    if len(states) == 0:
        raise ValueError(f"No valid samples found in {filepath}")
    
    states = np.array(states, dtype=np.float32)
    policies = np.array(policies, dtype=np.float32)
    values = np.array(values, dtype=np.float32)
    
    print(f"Loaded {len(states):,} samples")
    
    return states, policies, values


def load_multiple_jsonl(filepaths: List[str]) -> Tuple[np.ndarray, np.ndarray, np.ndarray]:
    """
    Load and concatenate data from multiple JSONL files.
    
    Useful for experience replay (combining multiple iterations).
    
    Args:
        filepaths: List of paths to .jsonl files
    
    Returns:
        states: (N, 10, 6, 6) numpy array
        policies: (N, 188) numpy array
        values: (N,) numpy array
    """
    all_states = []
    all_policies = []
    all_values = []
    
    for filepath in filepaths:
        states, policies, values = load_jsonl(filepath)
        all_states.append(states)
        all_policies.append(policies)
        all_values.append(values)
    
    states = np.concatenate(all_states, axis=0)
    policies = np.concatenate(all_policies, axis=0)
    values = np.concatenate(all_values, axis=0)
    
    print(f"Total samples from {len(filepaths)} files: {len(states):,}")
    
    return states, policies, values


def create_data_loader(
    states: np.ndarray,
    policies: np.ndarray,
    values: np.ndarray,
    batch_size: int = 256,
    shuffle: bool = True,
    num_workers: int = 0
) -> DataLoader:
    """
    Create a PyTorch DataLoader from training data.
    
    Args:
        states: (N, 10, 6, 6) numpy array
        policies: (N, 188) numpy array
        values: (N,) numpy array
        batch_size: Batch size for training
        shuffle: Whether to shuffle data
        num_workers: Number of worker processes for data loading
    
    Returns:
        DataLoader instance
    """
    dataset = BoopDataset(states, policies, values)
    
    loader = DataLoader(
        dataset,
        batch_size=batch_size,
        shuffle=shuffle,
        num_workers=num_workers,
        pin_memory=True  # Faster GPU transfer
    )
    
    return loader


def validate_data(states: np.ndarray, policies: np.ndarray, values: np.ndarray):
    """
    Print statistics about loaded data for sanity checking.
    
    Args:
        states: (N, 10, 6, 6) numpy array
        policies: (N, 188) numpy array
        values: (N,) numpy array
    """
    print("\n" + "="*60)
    print("DATA VALIDATION")
    print("="*60)
    
    print(f"\nDataset size: {len(states):,} samples")
    
    # State statistics
    print(f"\nState statistics:")
    print(f"  Shape: {states.shape}")
    print(f"  Range: [{states.min():.3f}, {states.max():.3f}]")
    print(f"  Mean: {states.mean():.3f}")
    print(f"  Std: {states.std():.3f}")
    
    # Policy statistics
    print(f"\nPolicy statistics:")
    print(f"  Shape: {policies.shape}")
    print(f"  Policy sum range: [{policies.sum(axis=1).min():.4f}, {policies.sum(axis=1).max():.4f}]")
    print(f"  Average max probability: {policies.max(axis=1).mean():.4f}")
    print(f"  Average entropy: {compute_entropy(policies):.4f}")
    
    # Value statistics
    print(f"\nValue statistics:")
    print(f"  Shape: {values.shape}")
    print(f"  Range: [{values.min():.3f}, {values.max():.3f}]")
    print(f"  Mean: {values.mean():.3f}")
    print(f"  Win rate: {(values > 0).sum() / len(values) * 100:.1f}%")
    print(f"  Loss rate: {(values < 0).sum() / len(values) * 100:.1f}%")
    print(f"  Draw rate: {(values == 0).sum() / len(values) * 100:.1f}%")
    
    # Warnings
    print(f"\nData quality checks:")
    
    if not np.allclose(policies.sum(axis=1), 1.0, atol=1e-4):
        print("  ⚠ WARNING: Some policies don't sum to 1.0")
    else:
        print("  ✓ All policies sum to 1.0")
    
    if np.any(values < -1.0) or np.any(values > 1.0):
        print("  ⚠ WARNING: Some values are outside [-1, 1]")
    else:
        print("  ✓ All values in valid range [-1, 1]")
    
    if np.abs(values.mean()) > 0.2:
        print(f"  ⚠ WARNING: Value mean is {values.mean():.3f}, might be imbalanced")
    else:
        print("  ✓ Value distribution looks balanced")
    
    print("="*60 + "\n")


def compute_entropy(policies: np.ndarray) -> float:
    """
    Compute average entropy of policy distributions.
    
    Higher entropy = more uniform/uncertain policies
    Lower entropy = more peaked/confident policies
    
    Args:
        policies: (N, 188) numpy array
    
    Returns:
        Average entropy across all samples
    """
    # Avoid log(0) by adding small epsilon
    eps = 1e-8
    entropies = -np.sum(policies * np.log(policies + eps), axis=1)
    return entropies.mean()


def save_preprocessed(
    states: np.ndarray,
    policies: np.ndarray,
    values: np.ndarray,
    filepath: str
):
    """
    Save preprocessed data to .npz file for faster loading.
    
    Args:
        states: (N, 10, 6, 6) numpy array
        policies: (N, 188) numpy array
        values: (N,) numpy array
        filepath: Path to save .npz file
    """
    filepath = Path(filepath)
    filepath.parent.mkdir(parents=True, exist_ok=True)
    
    np.savez_compressed(
        filepath,
        states=states,
        policies=policies,
        values=values
    )
    
    print(f"Saved preprocessed data to {filepath}")


def load_preprocessed(filepath: str) -> Tuple[np.ndarray, np.ndarray, np.ndarray]:
    """
    Load preprocessed data from .npz file.
    
    Args:
        filepath: Path to .npz file
    
    Returns:
        states: (N, 10, 6, 6) numpy array
        policies: (N, 188) numpy array
        values: (N,) numpy array
    """
    data = np.load(filepath)
    return data['states'], data['policies'], data['values']


if __name__ == "__main__":
    # Example usage and testing
    import sys
    
    if len(sys.argv) > 1:
        # Load actual data file
        filepath = sys.argv[1]
        print(f"Loading {filepath}...\n")
        
        states, policies, values = load_jsonl(filepath)
        validate_data(states, policies, values)
        
        # Test DataLoader
        print("Creating DataLoader...")
        loader = create_data_loader(states, policies, values, batch_size=32)
        
        print(f"DataLoader created with {len(loader)} batches")
        
        # Test one batch
        state_batch, policy_batch, value_batch = next(iter(loader))
        print(f"\nSample batch:")
        print(f"  States: {state_batch.shape}")
        print(f"  Policies: {policy_batch.shape}")
        print(f"  Values: {value_batch.shape}")
        
    else:
        print("Usage: python data.py <path_to_jsonl_file>")
        print("\nOr create synthetic test data:")
        
        # Create synthetic test data
        print("\nCreating synthetic test data...")
        n_samples = 1000
        
        states = np.random.randn(n_samples, 10, 6, 6).astype(np.float32)
        
        # Random policies (normalized)
        policies = np.random.rand(n_samples, 188).astype(np.float32)
        policies = policies / policies.sum(axis=1, keepdims=True)
        
        # Random values
        values = np.random.choice([-1.0, 0.0, 1.0], size=n_samples).astype(np.float32)
        
        validate_data(states, policies, values)
        
        # Test DataLoader
        loader = create_data_loader(states, policies, values, batch_size=64)
        print(f"\nDataLoader test: {len(loader)} batches")
        
        state_batch, policy_batch, value_batch = next(iter(loader))
        print(f"Batch shapes: states={state_batch.shape}, policies={policy_batch.shape}, values={value_batch.shape}")
        
        print("\n✓ Data utilities test passed!")