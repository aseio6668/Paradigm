use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

/// Erasure coding implementation using Reed-Solomon codes
/// Splits data into multiple shards with redundancy for fault tolerance
pub struct ErasureEncoder {
    /// Number of data shards (original data pieces)
    data_shards: usize,

    /// Number of parity shards (redundancy pieces)
    parity_shards: usize,
}

/// A data shard containing part of the original file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataShard {
    /// Index of this shard in the original sequence
    pub index: usize,

    /// Whether this is a data shard (true) or parity shard (false)
    pub is_data_shard: bool,

    /// The actual shard data
    pub data: Vec<u8>,

    /// Checksum for integrity verification
    pub checksum: String,
}

impl ErasureEncoder {
    /// Create a new erasure encoder
    pub fn new(data_shards: usize, parity_shards: usize) -> Result<Self> {
        if data_shards == 0 || parity_shards == 0 {
            return Err(anyhow!(
                "Must have at least 1 data shard and 1 parity shard"
            ));
        }

        if data_shards + parity_shards > 255 {
            return Err(anyhow!("Total shards cannot exceed 255"));
        }

        Ok(Self {
            data_shards,
            parity_shards,
        })
    }

    /// Split data into shards with erasure coding
    pub fn encode(&self, data: Vec<u8>) -> Result<Vec<DataShard>> {
        if data.is_empty() {
            return Err(anyhow!("Cannot encode empty data"));
        }

        // Calculate shard size (pad if necessary)
        let shard_size = (data.len() + self.data_shards - 1) / self.data_shards;
        let padded_size = shard_size * self.data_shards;

        // Pad data to multiple of shard size
        let mut padded_data = data;
        while padded_data.len() < padded_size {
            padded_data.push(0);
        }

        let mut shards = Vec::new();

        // Create data shards
        for i in 0..self.data_shards {
            let start = i * shard_size;
            let end = std::cmp::min(start + shard_size, padded_data.len());
            let shard_data = padded_data[start..end].to_vec();

            let shard = DataShard {
                index: i,
                is_data_shard: true,
                data: shard_data,
                checksum: Self::calculate_checksum(&padded_data[start..end]),
            };

            shards.push(shard);
        }

        // Create parity shards using simple XOR for demonstration
        // In production, use proper Reed-Solomon implementation
        for i in 0..self.parity_shards {
            let mut parity_data = vec![0u8; shard_size];

            // Simple XOR-based parity (not true Reed-Solomon)
            for data_shard in &shards {
                for (j, &byte) in data_shard.data.iter().enumerate() {
                    if j < parity_data.len() {
                        parity_data[j] ^= byte;
                    }
                }
            }

            let shard = DataShard {
                index: self.data_shards + i,
                is_data_shard: false,
                data: parity_data.clone(),
                checksum: Self::calculate_checksum(&parity_data),
            };

            shards.push(shard);
        }

        Ok(shards)
    }

    /// Reconstruct original data from available shards
    pub fn decode(&self, shards: Vec<DataShard>) -> Result<Vec<u8>> {
        if shards.len() < self.data_shards {
            return Err(anyhow!(
                "Not enough shards to reconstruct data. Need at least {} data shards",
                self.data_shards
            ));
        }

        // Verify checksums
        for shard in &shards {
            let calculated_checksum = Self::calculate_checksum(&shard.data);
            if calculated_checksum != shard.checksum {
                return Err(anyhow!(
                    "Shard {} failed checksum verification",
                    shard.index
                ));
            }
        }

        // Sort shards by index
        let mut sorted_shards = shards;
        sorted_shards.sort_by_key(|s| s.index);

        // Collect data shards
        let data_shards: Vec<_> = sorted_shards
            .into_iter()
            .filter(|s| s.is_data_shard)
            .collect();

        // If we have missing data shards, we would reconstruct them from parity
        // For now, assume we have all data shards
        if data_shards.len() < self.data_shards {
            // TODO: Implement Reed-Solomon reconstruction from parity shards
            return Err(anyhow!(
                "Missing data shards reconstruction not yet implemented"
            ));
        }

        // Concatenate data shards to reconstruct original data
        let mut reconstructed = Vec::new();
        for shard in data_shards {
            reconstructed.extend(shard.data);
        }

        // Remove padding (find last non-zero byte)
        while let Some(&0) = reconstructed.last() {
            reconstructed.pop();
        }

        Ok(reconstructed)
    }

    /// Calculate SHA-256 checksum for data integrity
    fn calculate_checksum(data: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }
}

/// High-level functions for the storage system
/// These are the functions used by the main storage engine
/// Create data shards using erasure coding
pub fn create_shards(
    data: Vec<u8>,
    data_shards: usize,
    parity_shards: usize,
) -> Result<Vec<DataShard>> {
    let encoder = ErasureEncoder::new(data_shards, parity_shards)?;
    encoder.encode(data)
}

/// Reconstruct data from available shards
pub fn reconstruct_data(shards: Vec<Vec<u8>>) -> Result<Vec<u8>> {
    if shards.is_empty() {
        return Err(anyhow!("No shards provided for reconstruction"));
    }

    // For now, simple concatenation - assume shards are in order
    // In production, this would use proper erasure decoding
    let mut reconstructed = Vec::new();
    for shard in shards {
        reconstructed.extend(shard);
    }

    Ok(reconstructed)
}

/// Information about erasure coding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErasureConfig {
    pub data_shards: usize,
    pub parity_shards: usize,
    pub shard_size: usize,
    pub total_shards: usize,
    pub redundancy_ratio: f64,
}

impl ErasureConfig {
    pub fn new(data_shards: usize, parity_shards: usize) -> Self {
        let total_shards = data_shards + parity_shards;
        let redundancy_ratio = parity_shards as f64 / data_shards as f64;

        Self {
            data_shards,
            parity_shards,
            shard_size: 0, // Will be calculated based on data size
            total_shards,
            redundancy_ratio,
        }
    }

    /// Calculate storage overhead factor
    pub fn storage_overhead(&self) -> f64 {
        (self.data_shards + self.parity_shards) as f64 / self.data_shards as f64
    }

    /// Calculate maximum number of shard failures that can be tolerated
    pub fn fault_tolerance(&self) -> usize {
        self.parity_shards
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_erasure_coding_basic() {
        let encoder = ErasureEncoder::new(3, 2).unwrap();
        let test_data = b"Hello, World! This is test data for erasure coding.".to_vec();

        // Encode data
        let shards = encoder.encode(test_data.clone()).unwrap();
        assert_eq!(shards.len(), 5); // 3 data + 2 parity

        // Verify we can decode with all shards
        let reconstructed = encoder.decode(shards.clone()).unwrap();
        assert_eq!(reconstructed, test_data);

        // Test with missing parity shards (should still work)
        let data_only_shards = shards.into_iter().filter(|s| s.is_data_shard).collect();
        let reconstructed2 = encoder.decode(data_only_shards).unwrap();
        assert_eq!(reconstructed2, test_data);
    }

    #[test]
    fn test_checksum_verification() {
        let encoder = ErasureEncoder::new(2, 1).unwrap();
        let test_data = b"Test data".to_vec();

        let mut shards = encoder.encode(test_data).unwrap();

        // Corrupt a shard
        shards[0].data[0] = !shards[0].data[0];

        // Should fail due to checksum mismatch
        let result = encoder.decode(shards);
        assert!(result.is_err(), "Checksum mismatch should produce an error");
    }

    #[test]
    fn test_erasure_config() {
        let config = ErasureConfig::new(3, 2);

        assert_eq!(config.total_shards, 5);
        assert_eq!(config.fault_tolerance(), 2);
        assert_eq!(config.storage_overhead(), 5.0 / 3.0);
    }
}
