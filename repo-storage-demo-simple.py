#!/usr/bin/env python3
"""
Paradigm Network Repository Storage Demo (Simple Version)
========================================================

This script demonstrates how to store a complete source code repository
structure on the Paradigm network using PAR tokens.
"""

import os
import json
import hashlib
import base64
from pathlib import Path

def calculate_repo_storage_cost(repo_path):
    """Calculate the cost to store a repository on Paradigm network"""
    total_size = 0
    file_count = 0
    
    print("Analyzing repository structure...")
    
    for root, dirs, files in os.walk(repo_path):
        # Skip common ignore patterns
        dirs[:] = [d for d in dirs if not d.startswith('.') and d not in ['node_modules', '__pycache__', 'target', 'snt-web']]
        
        for file in files:
            if not file.startswith('.') and not file.endswith(('.pyc', '.o', '.exe', '.dll')):
                filepath = os.path.join(root, file)
                try:
                    size = os.path.getsize(filepath)
                    total_size += size
                    file_count += 1
                    
                    # Show some files being processed
                    if file_count <= 10:
                        rel_path = os.path.relpath(filepath, repo_path)
                        print(f"  - {rel_path} ({size} bytes)")
                    elif file_count == 11:
                        print(f"  ... and {len([f for r, _, fs in os.walk(repo_path) for f in fs]) - 10} more files")
                        
                except OSError:
                    continue
    
    # Cost calculation (0.001 PAR per KB)
    base_cost_per_kb = 0.001
    cost_estimate = (total_size / 1024) * base_cost_per_kb
    
    return {
        "total_size_bytes": total_size,
        "total_size_kb": total_size / 1024,
        "file_count": file_count,
        "estimated_cost_par": round(cost_estimate, 6)
    }

def create_storage_chunks(repo_path, max_files=5):
    """Create storage chunks for demonstration"""
    chunks = []
    chunk_id = 1
    
    print("\nCreating storage chunks...")
    
    for root, dirs, files in os.walk(repo_path):
        dirs[:] = [d for d in dirs if not d.startswith('.') and d not in ['node_modules', '__pycache__', 'target', 'snt-web']]
        
        for file in files[:max_files]:  # Limit for demo
            if file.startswith('.') or file.endswith(('.pyc', '.o', '.exe')):
                continue
                
            filepath = os.path.join(root, file)
            rel_filepath = os.path.relpath(filepath, repo_path)
            
            try:
                with open(filepath, 'rb') as f:
                    content = f.read()
                
                # Skip large files for demo
                if len(content) > 50000:  # 50KB limit for demo
                    print(f"  Skipping large file: {rel_filepath} ({len(content)} bytes)")
                    continue
                
                file_hash = hashlib.sha256(content).hexdigest()
                
                chunk = {
                    "chunk_id": f"chunk_{chunk_id:03d}",
                    "path": rel_filepath,
                    "size": len(content),
                    "hash": file_hash[:16],  # Short hash for display
                    "storage_cost_par": max(0.001, (len(content) / 1024) * 0.001)
                }
                
                chunks.append(chunk)
                print(f"  Created chunk {chunk_id}: {rel_filepath} ({len(content)} bytes, {chunk['storage_cost_par']:.6f} PAR)")
                chunk_id += 1
                
                if chunk_id > max_files:
                    break
                    
            except (OSError, UnicodeDecodeError):
                continue
        
        if chunk_id > max_files:
            break
    
    return chunks

def main():
    print("Paradigm Network Repository Storage Demo")
    print("=" * 50)
    
    # Use current directory as example repo
    repo_path = "."
    repo_name = "paradigm-network-demo"
    
    print(f"Repository: {repo_name}")
    print(f"Path: {os.path.abspath(repo_path)}")
    print()
    
    # Calculate storage cost
    cost_analysis = calculate_repo_storage_cost(repo_path)
    print(f"\nStorage Analysis:")
    print(f"  Files found: {cost_analysis['file_count']}")
    print(f"  Total size: {cost_analysis['total_size_kb']:.2f} KB ({cost_analysis['total_size_bytes']:,} bytes)")
    print(f"  Estimated cost: {cost_analysis['estimated_cost_par']:.6f} PAR tokens")
    print()
    
    # Create sample chunks
    chunks = create_storage_chunks(repo_path, max_files=5)
    
    print(f"\nSample Storage Chunks Created: {len(chunks)}")
    total_chunk_cost = sum(chunk['storage_cost_par'] for chunk in chunks)
    print(f"Sample chunks cost: {total_chunk_cost:.6f} PAR tokens")
    print()
    
    # Show how to submit to network
    print("How to Store on Paradigm Network:")
    print("=" * 40)
    print("1. Create a wallet:")
    print('   paradigm-wallet.exe create repo_wallet')
    print()
    print("2. Fund your wallet with PAR tokens (from contributors or exchanges)")
    print()
    print("3. Submit storage tasks:")
    for i, chunk in enumerate(chunks[:3], 1):
        print(f'   Task {i}: Store {chunk["path"]} - Cost: {chunk["storage_cost_par"]:.6f} PAR')
    print()
    print("4. Contributors will store your files and you pay the storage costs")
    print()
    print("5. Access stored files via network API:")
    print("   GET http://127.0.0.1:8080/api/tasks/available")
    print("   (Find your repository chunks in completed tasks)")
    print()
    
    # Show retrieval info
    print("Data Retrieval:")
    print("- Anyone can access your public repository data")
    print("- Use task IDs to reference specific files")
    print("- Network replicates across multiple contributor nodes")
    print("- Permanent storage as long as network exists")
    
    print(f"\nRepository storage simulation complete!")

if __name__ == "__main__":
    main()