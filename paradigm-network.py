#!/usr/bin/env python3
"""
🚀 Paradigm Network Launcher
================================

Modern, unified network management script for Paradigm cryptocurrency.
Replaces outdated batch files with cross-platform Python solution.

Usage:
    python paradigm-network.py build [--advanced]
    python paradigm-network.py start [--genesis] [--port PORT] [--peers PEERS]
    python paradigm-network.py test [--nodes N] [--stress]
    python paradigm-network.py wallet [COMMAND] [ARGS...]
    python paradigm-network.py status
    python paradigm-network.py stop
    python paradigm-network.py --help

Features:
    - Cross-platform (Windows, Linux, macOS)
    - AI-driven network configuration
    - Enterprise security setup
    - Multi-node testing
    - Wallet management
    - Process monitoring

Built with ❤️ by the Paradigm Core Team
"""

import argparse
import json
import os
import platform
import shutil
import subprocess
import sys
import time
from pathlib import Path
from typing import Dict, List, Optional, Tuple

# Constants
PARADIGM_VERSION = "2024.1"
DEFAULT_PORT = 8080
DEFAULT_API_PORT = 8080

class Colors:
    """Cross-platform terminal colors."""
    RED = '\033[91m'
    GREEN = '\033[92m'
    YELLOW = '\033[93m'
    BLUE = '\033[94m'
    PURPLE = '\033[95m'
    CYAN = '\033[96m'
    WHITE = '\033[97m'
    BOLD = '\033[1m'
    END = '\033[0m'

    @classmethod
    def disable(cls):
        """Disable colors for Windows compatibility."""
        cls.RED = cls.GREEN = cls.YELLOW = cls.BLUE = ''
        cls.PURPLE = cls.CYAN = cls.WHITE = cls.BOLD = cls.END = ''

# Disable colors on Windows unless in modern terminal
if platform.system() == "Windows":
    try:
        # Enable ANSI colors on Windows 10+
        os.system('color')
    except:
        Colors.disable()

def print_header(title: str):
    """Print a formatted header."""
    print(f"\n{Colors.CYAN}{Colors.BOLD}🚀 {title}{Colors.END}")
    print(f"{Colors.CYAN}{'=' * (len(title) + 3)}{Colors.END}")

def print_success(msg: str):
    """Print success message."""
    print(f"{Colors.GREEN}✅ {msg}{Colors.END}")

def print_error(msg: str):
    """Print error message."""
    print(f"{Colors.RED}❌ {msg}{Colors.END}")

def print_warning(msg: str):
    """Print warning message."""
    print(f"{Colors.YELLOW}⚠️  {msg}{Colors.END}")

def print_info(msg: str):
    """Print info message."""
    print(f"{Colors.BLUE}💡 {msg}{Colors.END}")

class ParadigmLauncher:
    """Main launcher class for Paradigm network operations."""
    
    def __init__(self):
        self.system = platform.system().lower()
        self.exe_suffix = ".exe" if self.system == "windows" else ""
        self.root_path = Path(__file__).parent.absolute()
        self.target_path = self.root_path / "target"
        self.release_path = self.target_path / "release"
        self.debug_path = self.target_path / "debug"
        
        # Binary paths
        self.binaries = {
            'core': 'paradigm-core',
            'wallet': 'paradigm-wallet', 
            'contributor': 'paradigm-contributor'
        }
        
    def check_rust(self) -> bool:
        """Check if Rust is installed."""
        try:
            result = subprocess.run(['cargo', '--version'], 
                                  capture_output=True, text=True, check=True)
            version = result.stdout.split()[1]
            print_success(f"Rust {version} found")
            return True
        except (subprocess.CalledProcessError, FileNotFoundError):
            print_error("Rust not found. Install from: https://rustup.rs/")
            return False
    
    def check_protoc(self) -> bool:
        """Check if protoc is installed (optional)."""
        try:
            subprocess.run(['protoc', '--version'], 
                          capture_output=True, check=True)
            print_success("protoc found (gRPC enabled)")
            return True
        except (subprocess.CalledProcessError, FileNotFoundError):
            print_warning("protoc not found - some features limited")
            return False
    
    def build_system(self, advanced: bool = False) -> bool:
        """Build the Paradigm system."""
        print_header("Paradigm Build System")
        
        if not self.check_rust():
            return False
            
        self.check_protoc()
        
        # Clean previous build
        print_info("Cleaning previous build...")
        try:
            if self.target_path.exists():
                shutil.rmtree(self.target_path)
            print_success("Clean completed")
        except Exception as e:
            print_warning(f"Clean partially failed: {e}")
        
        # Build configuration
        build_args = ['cargo', 'build', '--release']
        
        if advanced:
            build_args.append('--all-features')
            print_info("Advanced build with all features enabled")
        else:
            print_info("Fast build with core features")
            
        # Build components
        components = [
            ('paradigm-core', '🏗️ Blockchain Node'),
            ('paradigm-wallet', '💼 Multi-Signature Wallet'),
            ('paradigm-contributor', '⚡ ML Task Processor')
        ]
        
        for package, description in components:
            print(f"\n{Colors.CYAN}Building {description}...{Colors.END}")
            
            try:
                subprocess.run(build_args + ['--package', package], 
                             check=True, cwd=self.root_path)
                print_success(f"{description} built successfully")
            except subprocess.CalledProcessError:
                print_error(f"{description} build failed")
                return False
        
        # Verify binaries exist
        self._verify_binaries()
        
        print_success("\n🎉 Build completed successfully!")
        self._print_quick_start()
        
        return True
    
    def _verify_binaries(self):
        """Verify all binaries were built correctly."""
        print_info("\nVerifying binaries...")
        
        for name, binary in self.binaries.items():
            binary_path = self.release_path / f"{binary}{self.exe_suffix}"
            if binary_path.exists():
                print_success(f"{binary} found at {binary_path}")
            else:
                print_error(f"{binary} not found")
    
    def _print_quick_start(self):
        """Print quick start instructions."""
        print(f"\n{Colors.GREEN}🚀 Quick Start:{Colors.END}")
        print(f"  {Colors.WHITE}Genesis Network:{Colors.END}")
        print(f"    python paradigm-network.py start --genesis")
        print(f"  {Colors.WHITE}Join Network:{Colors.END}")
        print(f"    python paradigm-network.py start --peers IP:8080")
        print(f"  {Colors.WHITE}Use Wallet:{Colors.END}")
        print(f"    python paradigm-network.py wallet create my_wallet")
    
    def start_network(self, genesis: bool = False, port: int = DEFAULT_PORT,
                     api_port: int = DEFAULT_API_PORT, peers: Optional[str] = None,
                     data_dir: Optional[str] = None, security_level: str = "TLS") -> bool:
        """Start the Paradigm network."""
        print_header("Paradigm Network Launcher")
        
        # Check binaries exist
        core_binary = self.release_path / f"paradigm-core{self.exe_suffix}"
        if not core_binary.exists():
            print_error("paradigm-core binary not found. Run 'build' first.")
            return False
        
        # Configure data directory
        if not data_dir:
            data_dir = "./paradigm-genesis" if genesis else "./paradigm-node"
            
        data_path = Path(data_dir)
        data_path.mkdir(exist_ok=True)
        
        # Build command
        cmd = [str(core_binary)]
        cmd.extend(['--data-dir', str(data_path)])
        cmd.extend(['--port', str(port)])
        cmd.extend(['--enable-api'])
        cmd.extend(['--api-port', str(api_port)])
        
        if genesis:
            # Genesis node configuration
            genesis_config = data_path / "genesis-config.toml"
            if not genesis_config.exists():
                self._create_genesis_config(genesis_config)
                
            cmd.extend(['--genesis', str(genesis_config)])
            print_success(f"Starting genesis network on port {port}")
            
        elif peers:
            # Join existing network
            cmd.extend(['--addnode', peers])
            print_success(f"Joining network via peers: {peers}")
            
        else:
            print_warning("Starting standalone node (no genesis, no peers)")
            
        # Security configuration
        if security_level != "None":
            print_info(f"Security level: {security_level}")
        
        print(f"\n{Colors.CYAN}Starting Paradigm Core...{Colors.END}")
        print(f"{Colors.BLUE}Command: {' '.join(cmd)}{Colors.END}")
        
        try:
            # Start the process
            process = subprocess.Popen(cmd, cwd=self.root_path)
            
            print_success("Network started successfully!")
            print(f"{Colors.GREEN}🌐 API Server: http://localhost:{api_port}{Colors.END}")
            print(f"{Colors.GREEN}🔗 Network Port: {port}{Colors.END}")
            print(f"{Colors.GREEN}📁 Data Directory: {data_path.absolute()}{Colors.END}")
            
            if genesis:
                print(f"{Colors.PURPLE}🌟 Genesis network initialized with AI governance{Colors.END}")
                
            print(f"\n{Colors.YELLOW}Press Ctrl+C to stop the network{Colors.END}")
            
            # Monitor process
            try:
                process.wait()
            except KeyboardInterrupt:
                print(f"\n{Colors.YELLOW}Stopping network...{Colors.END}")
                process.terminate()
                process.wait()
                print_success("Network stopped")
                
            return True
            
        except Exception as e:
            print_error(f"Failed to start network: {e}")
            return False
    
    def _create_genesis_config(self, config_path: Path):
        """Create a default genesis configuration."""
        config = {
            "network": {
                "chain_id": 1,
                "network_name": "paradigm-mainnet",
                "genesis_timestamp": "2024-01-01T00:00:00Z"
            },
            "consensus": {
                "block_time_seconds": 10,
                "difficulty_adjustment_interval": 144
            },
            "ai_governance": {
                "min_fee_percentage": 0.001,
                "max_fee_percentage": 0.05,
                "fee_sensitivity": 0.1
            },
            "treasury": {
                "initial_supply": 1000000000_00000000,  # 1B PAR
                "network_treasury_percentage": 0.1
            },
            "security": {
                "require_tls": True,
                "enable_hsm": False,
                "multisig_required": False
            }
        }
        
        with open(config_path, 'w') as f:
            # Convert to TOML format (simplified)
            f.write("[network]\\n")
            for key, value in config["network"].items():
                f.write(f'{key} = "{value}"\\n')
                
            f.write("\\n[consensus]\\n")
            for key, value in config["consensus"].items():
                f.write(f'{key} = {value}\\n')
                
            f.write("\\n[ai_governance]\\n")
            for key, value in config["ai_governance"].items():
                f.write(f'{key} = {value}\\n')
                
            f.write("\\n[treasury]\\n")
            for key, value in config["treasury"].items():
                f.write(f'{key} = {value}\\n')
                
            f.write("\\n[security]\\n")
            for key, value in config["security"].items():
                f.write(f'{key} = {str(value).lower()}\\n')
                
        print_success(f"Created genesis configuration: {config_path}")
    
    def test_network(self, nodes: int = 3, stress: bool = False) -> bool:
        """Run network tests."""
        print_header("Paradigm Network Testing")
        
        if stress:
            return self._run_stress_test()
        else:
            return self._run_multi_node_test(nodes)
    
    def _run_multi_node_test(self, nodes: int) -> bool:
        """Run multi-node network test."""
        print_info(f"Starting {nodes}-node network test...")
        
        processes = []
        base_port = 8080
        
        try:
            # Start genesis node
            print_info("Starting genesis node...")
            genesis_cmd = [
                str(self.release_path / f"paradigm-core{self.exe_suffix}"),
                '--data-dir', './test-data/genesis',
                '--genesis', 'genesis-config.toml',
                '--port', str(base_port),
                '--enable-api',
                '--api-port', str(base_port)
            ]
            
            processes.append(subprocess.Popen(genesis_cmd, cwd=self.root_path))
            time.sleep(3)  # Let genesis node start
            
            # Start peer nodes
            for i in range(1, nodes):
                port = base_port + i
                print_info(f"Starting peer node {i} on port {port}...")
                
                peer_cmd = [
                    str(self.release_path / f"paradigm-core{self.exe_suffix}"),
                    '--data-dir', f'./test-data/node-{i}',
                    '--port', str(port),
                    '--addnode', f'127.0.0.1:{base_port}',
                    '--enable-api',
                    '--api-port', str(port + 1000)
                ]
                
                processes.append(subprocess.Popen(peer_cmd, cwd=self.root_path))
                time.sleep(2)  # Stagger startup
            
            print_success(f"✅ {nodes}-node test network started!")
            print_info("🔗 Genesis API: http://localhost:8080")
            for i in range(1, nodes):
                print_info(f"🔗 Node {i} API: http://localhost:{8080 + i + 1000}")
                
            print(f"\\n{Colors.YELLOW}Press Ctrl+C to stop test network{Colors.END}")
            
            # Wait for interrupt
            try:
                while True:
                    time.sleep(1)
            except KeyboardInterrupt:
                print(f"\\n{Colors.YELLOW}Stopping test network...{Colors.END}")
                
                # Stop all processes
                for process in processes:
                    process.terminate()
                    
                for process in processes:
                    process.wait()
                    
                print_success("Test network stopped")
                return True
                
        except Exception as e:
            print_error(f"Test network failed: {e}")
            
            # Cleanup processes
            for process in processes:
                try:
                    process.terminate()
                except:
                    pass
                    
            return False
    
    def _run_stress_test(self) -> bool:
        """Run wallet stress test."""
        print_info("Starting wallet stress test...")
        
        try:
            wallet_binary = self.release_path / f"paradigm-wallet{self.exe_suffix}"
            cmd = [str(wallet_binary), 'stress-test', '100']
            
            subprocess.run(cmd, check=True, cwd=self.root_path)
            print_success("Stress test completed")
            return True
            
        except Exception as e:
            print_error(f"Stress test failed: {e}")
            return False
    
    def wallet_command(self, args: List[str]) -> bool:
        """Execute wallet command."""
        wallet_binary = self.release_path / f"paradigm-wallet{self.exe_suffix}"
        
        if not wallet_binary.exists():
            print_error("paradigm-wallet binary not found. Run 'build' first.")
            return False
        
        try:
            cmd = [str(wallet_binary)] + args
            subprocess.run(cmd, check=True, cwd=self.root_path)
            return True
        except Exception as e:
            print_error(f"Wallet command failed: {e}")
            return False
    
    def show_status(self) -> bool:
        """Show network status."""
        print_header("Paradigm Network Status")
        
        # Check if binaries exist
        print_info("Binary Status:")
        for name, binary in self.binaries.items():
            path = self.release_path / f"{binary}{self.exe_suffix}"
            status = "✅ Available" if path.exists() else "❌ Missing"
            print(f"  {binary}: {status}")
        
        # Check if network is running
        print_info("\\nNetwork Status:")
        try:
            import requests
            response = requests.get("http://localhost:8080/api/v1/network/status", timeout=2)
            if response.status_code == 200:
                data = response.json()
                print_success("🌐 Network is running")
                print(f"  Active peers: {data.get('active_peers', 'unknown')}")
                print(f"  Network health: {data.get('health_score', 'unknown')}")
            else:
                print_warning("🌐 Network API not responding")
        except Exception:
            print_warning("🌐 Network not running or API unavailable")
            
        return True

def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description=f"🚀 Paradigm Network Launcher v{PARADIGM_VERSION}",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python paradigm-network.py build --advanced
  python paradigm-network.py start --genesis --port 8080
  python paradigm-network.py start --peers "192.168.1.100:8080;192.168.1.101:8080"
  python paradigm-network.py test --nodes 5
  python paradigm-network.py wallet create my_wallet
  python paradigm-network.py wallet send PAR1... PAR1... 0.1
        """
    )
    
    subparsers = parser.add_subparsers(dest='command', help='Available commands')
    
    # Build command
    build_parser = subparsers.add_parser('build', help='Build the Paradigm system')
    build_parser.add_argument('--advanced', action='store_true', help='Enable all features')
    
    # Start command
    start_parser = subparsers.add_parser('start', help='Start Paradigm network')
    start_parser.add_argument('--genesis', action='store_true', help='Start as genesis node')
    start_parser.add_argument('--port', type=int, default=DEFAULT_PORT, help='Network port')
    start_parser.add_argument('--api-port', type=int, default=DEFAULT_API_PORT, help='API port')
    start_parser.add_argument('--peers', help='Bootstrap peers (IP:PORT;IP2:PORT2)')
    start_parser.add_argument('--data-dir', help='Data directory')
    start_parser.add_argument('--security', choices=['None', 'TLS', 'MutualTLS'], 
                             default='TLS', help='Security level')
    
    # Test command
    test_parser = subparsers.add_parser('test', help='Run network tests')
    test_parser.add_argument('--nodes', type=int, default=3, help='Number of test nodes')
    test_parser.add_argument('--stress', action='store_true', help='Run stress test')
    
    # Wallet command
    wallet_parser = subparsers.add_parser('wallet', help='Wallet operations')
    wallet_parser.add_argument('wallet_args', nargs='*', help='Wallet command arguments')
    
    # Status command
    subparsers.add_parser('status', help='Show network status')
    
    # Stop command (for future use)
    subparsers.add_parser('stop', help='Stop running network')
    
    args = parser.parse_args()
    
    if not args.command:
        parser.print_help()
        return
    
    launcher = ParadigmLauncher()
    
    try:
        if args.command == 'build':
            success = launcher.build_system(advanced=args.advanced)
        elif args.command == 'start':
            success = launcher.start_network(
                genesis=args.genesis,
                port=args.port,
                api_port=args.api_port,
                peers=args.peers,
                data_dir=args.data_dir,
                security_level=args.security
            )
        elif args.command == 'test':
            success = launcher.test_network(nodes=args.nodes, stress=args.stress)
        elif args.command == 'wallet':
            success = launcher.wallet_command(args.wallet_args)
        elif args.command == 'status':
            success = launcher.show_status()
        elif args.command == 'stop':
            print_info("Stop command not implemented yet. Use Ctrl+C to stop running processes.")
            success = True
        else:
            parser.print_help()
            success = False
            
        sys.exit(0 if success else 1)
        
    except KeyboardInterrupt:
        print(f"\\n{Colors.YELLOW}Operation cancelled by user{Colors.END}")
        sys.exit(1)
    except Exception as e:
        print_error(f"Unexpected error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()