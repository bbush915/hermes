"""
Training script for Boop AlphaZero.

This implements the AlphaZero training loop:
1. Load self-play data from Rust
2. Train the neural network
3. Save model checkpoints
4. Export to ONNX for Rust inference
"""

import argparse
import torch
import torch.nn.functional as F
from torch.utils.tensorboard import SummaryWriter
from pathlib import Path
import time
from datetime import datetime
import json

from network import BoopNetwork, create_network
from data import load_jsonl, load_multiple_jsonl, create_data_loader, validate_data


def get_best_device():
    """
    Automatically select the best available device.
    
    Returns:
        Device string: 'cuda', 'mps', or 'cpu'
    """
    if torch.cuda.is_available():
        return 'cuda'
    elif hasattr(torch.backends, 'mps') and torch.backends.mps.is_available():
        return 'mps'
    else:
        return 'cpu'


class Trainer:
    """
    Handles the training loop for Boop AlphaZero.
    """
    
    def __init__(
        self,
        model,
        device=None,
        learning_rate=0.001,
        weight_decay=1e-4,
        log_dir='runs'
    ):
        """
        Args:
            model: BoopNetwork instance
            device: 'cuda', 'mps', or 'cpu' (None for auto-detect)
            learning_rate: Initial learning rate
            weight_decay: L2 regularization strength
            log_dir: Directory for TensorBoard logs
        """
        if device is None:
            device = get_best_device()
        self.model = model.to(device)
        self.device = device
        
        self.optimizer = torch.optim.Adam(
            self.model.parameters(),
            lr=learning_rate,
            weight_decay=weight_decay
        )
        
        # Optional: learning rate scheduler
        # self.scheduler = torch.optim.lr_scheduler.CosineAnnealingLR(
        #     self.optimizer, T_max=epochs
        # )
        
        # TensorBoard writer
        timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
        self.writer = SummaryWriter(f'{log_dir}/boop_{timestamp}')
        
        self.global_step = 0
        
    def compute_loss(self, policy_logits, value_pred, policy_target, value_target):
        """
        Compute AlphaZero loss (policy + value).
        
        Args:
            policy_logits: (batch_size, 188) - raw network output
            value_pred: (batch_size,) - predicted values
            policy_target: (batch_size, 188) - MCTS policy distribution
            value_target: (batch_size,) - actual game outcomes
        
        Returns:
            total_loss, policy_loss, value_loss
        """
        # Policy loss: KL divergence between MCTS policy and network output
        # This is equivalent to cross-entropy with soft targets
        policy_loss = -torch.sum(
            policy_target * F.log_softmax(policy_logits, dim=1),
            dim=1
        ).mean()
        
        # Value loss: MSE between predicted value and actual outcome
        value_loss = F.mse_loss(value_pred, value_target)
        
        # Combined loss
        total_loss = policy_loss + value_loss
        
        return total_loss, policy_loss, value_loss
    
    def train_epoch(self, data_loader, epoch):
        """
        Train for one epoch.
        
        Args:
            data_loader: PyTorch DataLoader
            epoch: Current epoch number
        
        Returns:
            Dictionary of average losses
        """
        self.model.train()
        
        total_loss_sum = 0
        policy_loss_sum = 0
        value_loss_sum = 0
        num_batches = 0
        
        for batch_idx, (states, policies, values) in enumerate(data_loader):
            # Move to device
            states = states.to(self.device)
            policies = policies.to(self.device)
            values = values.to(self.device)
            
            # Forward pass
            policy_logits, value_pred = self.model(states)
            
            # Compute loss
            total_loss, policy_loss, value_loss = self.compute_loss(
                policy_logits, value_pred, policies, values
            )
            
            # Backward pass
            self.optimizer.zero_grad()
            total_loss.backward()
            self.optimizer.step()
            
            # Track losses
            total_loss_sum += total_loss.item()
            policy_loss_sum += policy_loss.item()
            value_loss_sum += value_loss.item()
            num_batches += 1
            
            # Log to TensorBoard
            self.writer.add_scalar('Loss/total_step', total_loss.item(), self.global_step)
            self.writer.add_scalar('Loss/policy_step', policy_loss.item(), self.global_step)
            self.writer.add_scalar('Loss/value_step', value_loss.item(), self.global_step)
            
            self.global_step += 1
            
            # Print progress
            if (batch_idx + 1) % 10 == 0:
                print(f'  Batch {batch_idx + 1}/{len(data_loader)} | '
                      f'Loss: {total_loss.item():.4f} '
                      f'(policy: {policy_loss.item():.4f}, value: {value_loss.item():.4f})')
        
        # Average losses
        avg_total_loss = total_loss_sum / num_batches
        avg_policy_loss = policy_loss_sum / num_batches
        avg_value_loss = value_loss_sum / num_batches
        
        # Log epoch averages
        self.writer.add_scalar('Loss/total_epoch', avg_total_loss, epoch)
        self.writer.add_scalar('Loss/policy_epoch', avg_policy_loss, epoch)
        self.writer.add_scalar('Loss/value_epoch', avg_value_loss, epoch)
        
        return {
            'total_loss': avg_total_loss,
            'policy_loss': avg_policy_loss,
            'value_loss': avg_value_loss
        }
    
    def evaluate_accuracy(self, data_loader):
        """
        Evaluate model accuracy on validation data.
        
        Args:
            data_loader: PyTorch DataLoader
        
        Returns:
            Dictionary of metrics
        """
        self.model.eval()
        
        value_correct = 0
        policy_top1_correct = 0
        total_samples = 0
        
        with torch.no_grad():
            for states, policies, values in data_loader:
                states = states.to(self.device)
                policies = policies.to(self.device)
                values = values.to(self.device)
                
                policy_logits, value_pred = self.model(states)
                
                # Value accuracy: correct sign prediction
                value_correct += ((value_pred > 0) == (values > 0)).sum().item()
                
                # Policy accuracy: top-1 match with MCTS
                policy_pred_top1 = policy_logits.argmax(dim=1)
                policy_target_top1 = policies.argmax(dim=1)
                policy_top1_correct += (policy_pred_top1 == policy_target_top1).sum().item()
                
                total_samples += len(states)
        
        return {
            'value_accuracy': value_correct / total_samples,
            'policy_top1_accuracy': policy_top1_correct / total_samples
        }
    
    def train(self, train_loader, num_epochs, val_loader=None):
        """
        Main training loop.
        
        Args:
            train_loader: Training data loader
            num_epochs: Number of epochs to train
            val_loader: Optional validation data loader
        
        Returns:
            Training history
        """
        print(f"\nTraining on {self.device}")
        print(f"Model parameters: {self.model.get_num_parameters():,}")
        print(f"Training samples: {len(train_loader.dataset):,}")
        print(f"Batches per epoch: {len(train_loader)}")
        print(f"Epochs: {num_epochs}\n")
        
        history = []
        
        for epoch in range(num_epochs):
            print(f"Epoch {epoch + 1}/{num_epochs}")
            
            start_time = time.time()
            
            # Train
            train_metrics = self.train_epoch(train_loader, epoch)
            
            # Validation
            if val_loader is not None:
                val_metrics = self.evaluate_accuracy(val_loader)
                print(f"  Validation - Value acc: {val_metrics['value_accuracy']:.4f}, "
                      f"Policy top-1 acc: {val_metrics['policy_top1_accuracy']:.4f}")
                
                self.writer.add_scalar('Accuracy/value', val_metrics['value_accuracy'], epoch)
                self.writer.add_scalar('Accuracy/policy_top1', val_metrics['policy_top1_accuracy'], epoch)
            
            epoch_time = time.time() - start_time
            
            print(f"  Epoch completed in {epoch_time:.1f}s | "
                  f"Total loss: {train_metrics['total_loss']:.4f}\n")
            
            history.append({
                'epoch': epoch,
                'train': train_metrics,
                'val': val_metrics if val_loader else None,
                'time': epoch_time
            })
        
        print("Training completed!")
        return history
    
    def save_checkpoint(self, filepath, epoch=None, metadata=None):
        """
        Save model checkpoint.
        
        Args:
            filepath: Path to save checkpoint
            epoch: Current epoch (optional)
            metadata: Additional metadata to save (optional)
        """
        filepath = Path(filepath)
        filepath.parent.mkdir(parents=True, exist_ok=True)
        
        checkpoint = {
            'model_state_dict': self.model.state_dict(),
            'optimizer_state_dict': self.optimizer.state_dict(),
            'epoch': epoch,
            'global_step': self.global_step,
        }
        
        if metadata:
            checkpoint['metadata'] = metadata
        
        torch.save(checkpoint, filepath)
        print(f"Saved checkpoint to {filepath}")
    
    def load_checkpoint(self, filepath):
        """
        Load model checkpoint.
        
        Args:
            filepath: Path to checkpoint file
        """
        checkpoint = torch.load(filepath, map_location=self.device)
        
        self.model.load_state_dict(checkpoint['model_state_dict'])
        self.optimizer.load_state_dict(checkpoint['optimizer_state_dict'])
        self.global_step = checkpoint.get('global_step', 0)
        
        print(f"Loaded checkpoint from {filepath}")
        return checkpoint


def export_to_onnx(model, filepath, opset_version=11):
    """
    Export model to ONNX format for Rust inference.
    
    Args:
        model: Trained BoopNetwork
        filepath: Path to save .onnx file
        opset_version: ONNX opset version
    """
    filepath = Path(filepath)
    filepath.parent.mkdir(parents=True, exist_ok=True)
    
    model = model.cpu()
    model.eval()
    
    # Create dummy input on CPU
    dummy_input = torch.randn(1, 10, 6, 6)
    
    # Export
    torch.onnx.export(
        model,
        dummy_input,
        filepath,
        export_params=True,
        opset_version=opset_version,
        do_constant_folding=True,
        input_names=['state'],
        output_names=['policy', 'value'],
        dynamic_axes={
            'state': {0: 'batch_size'},
            'policy': {0: 'batch_size'},
            'value': {0: 'batch_size'}
        }
    )
    
    print(f"Exported ONNX model to {filepath}")


def main():
    parser = argparse.ArgumentParser(description='Train Boop AlphaZero network')
    
    # Data
    parser.add_argument('--data', type=str, required=True,
                       help='Path to training data (.jsonl file)')
    parser.add_argument('--replay', type=str, nargs='+',
                       help='Additional data files for experience replay')
    
    # Model
    parser.add_argument('--blocks', type=int, default=8,
                       help='Number of residual blocks (default: 8)')
    parser.add_argument('--channels', type=int, default=64,
                       help='Number of channels (default: 64)')
    parser.add_argument('--load', type=str,
                       help='Load checkpoint to resume training')
    
    # Training
    parser.add_argument('--epochs', type=int, default=10,
                       help='Number of epochs (default: 10)')
    parser.add_argument('--batch-size', type=int, default=256,
                       help='Batch size (default: 256)')
    parser.add_argument('--lr', type=float, default=0.001,
                       help='Learning rate (default: 0.001)')
    parser.add_argument('--weight-decay', type=float, default=1e-4,
                       help='Weight decay (default: 1e-4)')
    
    # Output
    parser.add_argument('--output', type=str, default='models/checkpoints/model.pt',
                       help='Output checkpoint path')
    parser.add_argument('--onnx', type=str,
                       help='Export ONNX model path (optional)')
    parser.add_argument('--log-dir', type=str, default='runs',
                       help='TensorBoard log directory')
    
    # Device
    parser.add_argument('--device', type=str, default='auto',
                       choices=['auto', 'cuda', 'mps', 'cpu'],
                       help='Device to use (default: auto)')
    
    args = parser.parse_args()
    
    # Set device
    if args.device == 'auto':
        device = get_best_device()
    else:
        device = args.device
    
    print("="*60)
    print("BOOP ALPHAZERO TRAINING")
    print("="*60)
    
    # Load data
    print(f"\nLoading training data...")
    if args.replay:
        filepaths = [args.data] + args.replay
        states, policies, values = load_multiple_jsonl(filepaths)
    else:
        states, policies, values = load_jsonl(args.data)
    
    validate_data(states, policies, values)
    
    # Create data loader
    train_loader = create_data_loader(
        states, policies, values,
        batch_size=args.batch_size,
        shuffle=True
    )
    
    # Create model
    print(f"\nCreating network...")
    if args.load:
        # Load existing model
        config = {'num_blocks': args.blocks, 'num_channels': args.channels}
        model = create_network(config)
        trainer = Trainer(model, device=device, learning_rate=args.lr,
                         weight_decay=args.weight_decay, log_dir=args.log_dir)
        checkpoint = trainer.load_checkpoint(args.load)
        start_epoch = checkpoint.get('epoch', 0) + 1
    else:
        # Create new model
        config = {'num_blocks': args.blocks, 'num_channels': args.channels}
        model = create_network(config)
        trainer = Trainer(model, device=device, learning_rate=args.lr,
                         weight_decay=args.weight_decay, log_dir=args.log_dir)
        start_epoch = 0
    
    # Train
    print(f"\nStarting training...")
    history = trainer.train(train_loader, args.epochs)
    
    # Save checkpoint
    print(f"\nSaving checkpoint...")
    metadata = {
        'config': config,
        'args': vars(args),
        'final_epoch': start_epoch + args.epochs,
        'history': history
    }
    trainer.save_checkpoint(args.output, epoch=start_epoch + args.epochs - 1, metadata=metadata)
    
    # Export ONNX
    if args.onnx:
        print(f"\nExporting to ONNX...")
        export_to_onnx(model, args.onnx)
    
    # Save training history
    history_path = Path(args.output).parent / 'training_history.json'
    with open(history_path, 'w') as f:
        json.dump(history, f, indent=2)
    print(f"Saved training history to {history_path}")
    
    print("\n" + "="*60)
    print("TRAINING COMPLETE!")
    print("="*60)
    print(f"\nCheckpoint: {args.output}")
    if args.onnx:
        print(f"ONNX model: {args.onnx}")
    print(f"\nView training progress: tensorboard --logdir {args.log_dir}")


if __name__ == "__main__":
    main()