#!/usr/bin/env python3
"""
Paradigm Network Repository Storage Demo
========================================

This script demonstrates how to store a complete source code repository
structure on the Paradigm network using PAR tokens.

Author: Claude Code Assistant
"""

import os
import json
import hashlib
import base64
import requests
from pathlib import Path
from typing import Dict, List, Any

class ParadigmRepoStorage:
    def __init__(self, node_url: str = "http://127.0.0.1:8080"):
        self.node_url = node_url
        self.api_base = f"{node_url}/api"
        
    def calculate_storage_cost(self, repo_path: str) -> Dict[str, Any]:
        """Calculate the cost to store a repository"""
        total_size = 0
        file_count = 0
        
        for root, dirs, files in os.walk(repo_path):
            # Skip common ignore patterns
            dirs[:] = [d for d in dirs if not d.startswith('.') and d not in ['node_modules', '__pycache__', 'target']]
            
            for file in files:
                if not file.startswith('.') and not file.endswith(('.pyc', '.o', '.exe')):
                    filepath = os.path.join(root, file)
                    try:
                        size = os.path.getsize(filepath)
                        total_size += size
                        file_count += 1
                    except OSError:
                        continue
        
        # Cost calculation
        base_cost_per_kb = 0.001  # PAR tokens per KB
        cost_estimate = (total_size / 1024) * base_cost_per_kb
        
        return {
            "total_size_bytes": total_size,
            "total_size_kb": total_size / 1024,
            "file_count": file_count,
            "estimated_cost_par": round(cost_estimate, 6),
            "storage_method": "chunked_tasks"
        }
    
    def create_repository_manifest(self, repo_path: str, repo_name: str) -> Dict[str, Any]:
        """Create a repository manifest with metadata"""
        repo_path = Path(repo_path)
        
        manifest = {
            "repository": {
                "name": repo_name,
                "version": "1.0.0",
                "created_at": "2025-09-02T13:45:00Z",
                "storage_method": "paradigm_chunked",
                "description": f"Repository stored on Paradigm Network"
            },
            "structure": {},
            "files": {},
            "chunks": []
        }
        
        chunk_id = 1
        
        for root, dirs, files in os.walk(repo_path):
            # Skip ignored directories
            dirs[:] = [d for d in dirs if not d.startswith('.') and d not in ['node_modules', '__pycache__', 'target', '.git']]
            
            rel_root = os.path.relpath(root, repo_path)
            if rel_root == '.':
                rel_root = ''
            
            # Add directory to structure
            if rel_root:
                manifest["structure"][rel_root] = "directory"
            
            for file in files:
                if file.startswith('.') or file.endswith(('.pyc', '.o', '.exe', '.dll')):
                    continue
                    
                filepath = os.path.join(root, file)
                rel_filepath = os.path.relpath(filepath, repo_path)
                
                try:
                    with open(filepath, 'rb') as f:
                        content = f.read()
                    
                    # Calculate hash
                    file_hash = hashlib.sha256(content).hexdigest()
                    
                    # Encode content
                    encoded_content = base64.b64encode(content).decode('utf-8')
                    
                    # Create file entry
                    manifest["files"][rel_filepath] = {
                        "size": len(content),
                        "hash": file_hash,
                        "chunk_id": f"chunk_{chunk_id:03d}",
                        "encoding": "base64"
                    }
                    
                    # Create storage chunk
                    chunk = {
                        "chunk_id": f"chunk_{chunk_id:03d}",
                        "task_type": "repository_file_storage",
                        "path": rel_filepath,
                        "content": encoded_content,
                        "metadata": {
                            "size": len(content),
                            "hash": file_hash,
                            "mime_type": self.guess_mime_type(file)
                        }
                    }
                    
                    manifest["chunks"].append(chunk)
                    chunk_id += 1
                    
                except (OSError, UnicodeDecodeError):
                    print(f"Skipping unreadable file: {rel_filepath}")
                    continue
        
        return manifest
    
    def guess_mime_type(self, filename: str) -> str:
        """Guess MIME type from filename"""
        ext_map = {
            '.py': 'text/x-python',
            '.js': 'application/javascript',
            '.html': 'text/html',
            '.css': 'text/css',
            '.rs': 'text/x-rust',
            '.cpp': 'text/x-c++src',
            '.h': 'text/x-chdr',
            '.json': 'application/json',
            '.md': 'text/markdown',
            '.txt': 'text/plain',
        }
        
        ext = Path(filename).suffix.lower()
        return ext_map.get(ext, 'application/octet-stream')
    
    def submit_repository_to_network(self, manifest: Dict[str, Any]) -> Dict[str, Any]:
        """Submit repository chunks to the Paradigm network as tasks"""
        results = {
            "manifest_hash": hashlib.sha256(json.dumps(manifest, sort_keys=True).encode()).hexdigest(),
            "submitted_chunks": [],
            "failed_chunks": [],
            "total_cost": 0.0
        }
        
        print(f"📦 Submitting {len(manifest['chunks'])} chunks to Paradigm network...")
        
        for i, chunk in enumerate(manifest["chunks"]):
            print(f"📤 Submitting chunk {i+1}/{len(manifest['chunks'])}: {chunk['path']}")
            
            # Create task submission
            task_data = {
                "task_id": f"repo_storage_{chunk['chunk_id']}",
                "task_type": "repository_file_storage", 
                "difficulty": 1,
                "data": json.dumps(chunk),
                "reward": self.calculate_chunk_reward(len(chunk["content"])),
                "timestamp": 1725282300  # Current timestamp
            }
            
            try:
                # Submit to network API (simulated)
                success = self.submit_task(task_data)
                
                if success:
                    results["submitted_chunks"].append({
                        "chunk_id": chunk["chunk_id"],
                        "path": chunk["path"],
                        "size": chunk["metadata"]["size"],
                        "cost": task_data["reward"] / 1000000000  # Convert to PAR
                    })
                    results["total_cost"] += task_data["reward"] / 1000000000
                else:
                    results["failed_chunks"].append(chunk["chunk_id"])
                    
            except Exception as e:
                print(f"❌ Failed to submit {chunk['path']}: {str(e)}")
                results["failed_chunks"].append(chunk["chunk_id"])
        
        return results
    
    def calculate_chunk_reward(self, content_size: int) -> int:
        """Calculate reward in smallest PAR units for storing a chunk"""
        # Base cost: 0.001 PAR per KB
        kb_size = content_size / 1024
        par_cost = max(0.001, kb_size * 0.001)  # Minimum 0.001 PAR
        return int(par_cost * 1000000000)  # Convert to smallest units
    
    def submit_task(self, task_data: Dict[str, Any]) -> bool:
        """Submit a task to the Paradigm network"""
        try:
            # In a real implementation, this would POST to the API
            # For demo purposes, we'll simulate success
            url = f"{self.api_base}/tasks/submit"
            
            # Simulate network call (replace with actual HTTP request)
            print(f"  → Simulating POST to {url}")
            print(f"  → Task size: {len(task_data['data'])} bytes")
            print(f"  → Reward: {task_data['reward'] / 1000000000:.6f} PAR")
            
            # Simulate success (90% success rate)
            import random
            return random.random() > 0.1
            
        except Exception as e:
            print(f"Network error: {str(e)}")
            return False

def main():
    """Demo: Store a source code repository on Paradigm network"""
    print("🌟 Paradigm Network Repository Storage Demo")
    print("=" * 50)
    
    # Initialize storage client
    storage = ParadigmRepoStorage()
    
    # Example repository path (use current directory as demo)
    repo_path = "."
    repo_name = "paradigm-network-demo"
    
    print(f"📁 Analyzing repository: {repo_name}")
    
    # Calculate storage cost
    cost_analysis = storage.calculate_storage_cost(repo_path)
    print(f"📊 Storage Analysis:")
    print(f"   • Files: {cost_analysis['file_count']}")
    print(f"   • Size: {cost_analysis['total_size_kb']:.2f} KB")
    print(f"   • Estimated cost: {cost_analysis['estimated_cost_par']:.6f} PAR tokens")
    print()
    
    # Create repository manifest
    print("📋 Creating repository manifest...")
    manifest = storage.create_repository_manifest(repo_path, repo_name)
    
    print(f"✅ Manifest created:")
    print(f"   • Repository: {manifest['repository']['name']}")
    print(f"   • Files: {len(manifest['files'])}")
    print(f"   • Chunks: {len(manifest['chunks'])}")
    print()
    
    # Simulate submission to network
    print("🚀 Submitting to Paradigm network...")
    results = storage.submit_repository_to_network(manifest)
    
    print(f"📈 Submission Results:")
    print(f"   • Manifest hash: {results['manifest_hash'][:16]}...")
    print(f"   • Successful chunks: {len(results['submitted_chunks'])}")
    print(f"   • Failed chunks: {len(results['failed_chunks'])}")
    print(f"   • Total cost: {results['total_cost']:.6f} PAR tokens")
    print()
    
    print("🎉 Repository storage simulation complete!")
    print("\n💡 To actually store on the network:")
    print("   1. Ensure you have a Paradigm wallet with PAR tokens")
    print("   2. Run: paradigm-wallet.exe create repo_wallet")
    print("   3. Use the network API to submit tasks")
    print("   4. Monitor task completion via contributors")

if __name__ == "__main__":
    main()