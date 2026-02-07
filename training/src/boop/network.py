"""
Neural Network for Boop AlphaZero

Architecture:
- Input: 10 planes of 6x6 (from BoopStateEncoder)
- Body: Convolutional tower with residual blocks
- Policy head: 188 action logits (from BoopActionEncoder)
- Value head: Single scalar in [-1, 1]
"""

import torch
import torch.nn as nn
import torch.nn.functional as F
import numpy as np


class ResidualBlock(nn.Module):
    """
    Standard residual block with two conv layers.
    
    x -> Conv -> BN -> ReLU -> Conv -> BN -> (+) -> ReLU
    |______________________________________________|
    """
    
    def __init__(self, num_channels: int):
        super().__init__()
        self.conv1 = nn.Conv2d(num_channels, num_channels, kernel_size=3, padding=1, bias=False)
        self.bn1 = nn.BatchNorm2d(num_channels)
        self.conv2 = nn.Conv2d(num_channels, num_channels, kernel_size=3, padding=1, bias=False)
        self.bn2 = nn.BatchNorm2d(num_channels)
        
    def forward(self, x: torch.Tensor) -> torch.Tensor:
        residual = x
        
        out = self.conv1(x)
        out = self.bn1(out)
        out = F.relu(out)
        
        out = self.conv2(out)
        out = self.bn2(out)
        
        out += residual
        out = F.relu(out)
        
        return out


class BoopNetwork(nn.Module):
    """
    AlphaZero-style network for Boop.
    
    Based on your Rust encodings:
    - State: 10 channels x 6x6 board
    - Actions: 188 total (72 placements + 80 three-in-row + 36 single graduates)
    
    Args:
        num_blocks: Number of residual blocks (5-12 recommended for Boop)
        num_channels: Channels in conv layers (64-128 recommended)
    """
    
    def __init__(self, num_blocks: int = 8, num_channels: int = 64):
        super().__init__()
        
        # Constants from your Rust code
        self.board_size = 6
        self.input_channels = 10  # From BoopStateEncoder
        self.num_actions = 188     # From BoopActionEncoder
        
        # Initial convolution
        self.conv_input = nn.Conv2d(
            self.input_channels,
            num_channels,
            kernel_size=3,
            padding=1,
            bias=False
        )
        self.bn_input = nn.BatchNorm2d(num_channels)
        
        # Residual tower
        self.residual_blocks = nn.ModuleList([
            ResidualBlock(num_channels) for _ in range(num_blocks)
        ])
        
        # Policy head
        self.policy_conv = nn.Conv2d(num_channels, 32, kernel_size=1, bias=False)
        self.policy_bn = nn.BatchNorm2d(32)
        self.policy_fc = nn.Linear(32 * self.board_size * self.board_size, self.num_actions)
        
        # Value head
        self.value_conv = nn.Conv2d(num_channels, 32, kernel_size=1, bias=False)
        self.value_bn = nn.BatchNorm2d(32)
        self.value_fc1 = nn.Linear(32 * self.board_size * self.board_size, 64)
        self.value_fc2 = nn.Linear(64, 1)
        
    def forward(self, x: torch.Tensor):
        """
        Forward pass.
        
        Args:
            x: Tensor of shape (batch_size, 10, 6, 6)
        
        Returns:
            policy_logits: Tensor of shape (batch_size, 188)
            value: Tensor of shape (batch_size,) in range [-1, 1]
        """
        # Input processing
        x = self.conv_input(x)
        x = self.bn_input(x)
        x = F.relu(x)
        
        # Residual tower
        for block in self.residual_blocks:
            x = block(x)
        
        # Policy head
        policy = self.policy_conv(x)
        policy = self.policy_bn(policy)
        policy = F.relu(policy)
        policy = policy.view(policy.size(0), -1)  # Flatten
        policy_logits = self.policy_fc(policy)
        
        # Value head
        value = self.value_conv(x)
        value = self.value_bn(value)
        value = F.relu(value)
        value = value.view(value.size(0), -1)  # Flatten
        value = F.relu(self.value_fc1(value))
        value = torch.tanh(self.value_fc2(value))  # Output in [-1, 1]
        value = value.squeeze(-1)  # Shape: (batch_size,)
        
        return policy_logits, value
    
    def predict(self, state: torch.Tensor | np.ndarray) -> tuple[np.ndarray, float]:
        """
        Predict for a single state (convenience method).
        
        Args:
            state: numpy array of shape (10, 6, 6) or torch tensor
        
        Returns:
            policy: numpy array of shape (188,) - probability distribution
            value: float in [-1, 1]
        """
        self.eval()
        with torch.no_grad():
            if not isinstance(state, torch.Tensor):
                state = torch.from_numpy(state).float()
            
            # Add batch dimension
            state = state.unsqueeze(0)
            
            policy_logits, value = self.forward(state)
            
            # Convert logits to probabilities
            policy = F.softmax(policy_logits, dim=1)
            
            # Remove batch dimension and convert to numpy
            policy = policy.squeeze(0).cpu().numpy()
            value = value.item()
            
        return policy, value
    
    def get_num_parameters(self):
        """Count total parameters in the network."""
        return sum(p.numel() for p in self.parameters())
    
    def get_num_trainable_parameters(self):
        """Count trainable parameters in the network."""
        return sum(p.numel() for p in self.parameters() if p.requires_grad)


def create_network(config: dict[str, int] | None = None):
    """
    Factory function to create a network with config.
    
    Args:
        config: dict with 'num_blocks' and 'num_channels', or None for defaults
    
    Returns:
        BoopNetwork instance
    """
    if config is None:
        config = {}
    
    num_blocks = config.get('num_blocks', 8)
    num_channels = config.get('num_channels', 64)
    
    network = BoopNetwork(num_blocks=num_blocks, num_channels=num_channels)
    
    print(f"Created BoopNetwork:")
    print(f"  Residual blocks: {num_blocks}")
    print(f"  Channels: {num_channels}")
    print(f"  Total parameters: {network.get_num_parameters():,}")
    
    return network


if __name__ == "__main__":
    # Quick test
    import numpy as np
    
    print("Testing BoopNetwork...\n")
    
    # Create network
    net = create_network({'num_blocks': 8, 'num_channels': 64})
    
    # Test with random input
    batch_size = 4
    dummy_input = torch.randn(batch_size, 10, 6, 6)
    
    print(f"\nInput shape: {dummy_input.shape}")
    
    policy_logits, value = net(dummy_input)
    
    print(f"Policy logits shape: {policy_logits.shape}")
    print(f"Value shape: {value.shape}")
    print(f"Value range: [{value.min().item():.3f}, {value.max().item():.3f}]")
    
    # Test single prediction
    print("\nTesting single prediction...")
    single_state = np.random.randn(10, 6, 6).astype(np.float32)
    policy, value = net.predict(single_state)
    
    print(f"Policy shape: {policy.shape}")
    print(f"Policy sum (should be ~1.0): {policy.sum():.4f}")
    print(f"Value: {value:.3f}")
    
    print("\n✓ Network test passed!")