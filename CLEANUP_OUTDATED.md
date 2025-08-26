# 🧹 Paradigm Cleanup Guide

This guide identifies outdated scripts and files that should be removed to modernize the Paradigm repository.

## 📋 Scripts to Remove

### ❌ **Outdated Batch Files** 
*(Replaced by `paradigm-network.py`)*

```
genesis.bat                    # Complex legacy genesis launcher
launch-genesis.bat            # Outdated genesis script
build-genesis-only.bat        # Redundant build script
test-local-network.bat        # Basic local test (replaced)
contributor-connect.bat       # Simple contributor launcher
setup-release.bat            # Legacy release setup
```

### ✅ **Keep These Files**
*(Still useful or recently updated)*

```
build.bat                     # Fast Windows build (still useful)
build-advanced.bat           # Advanced Windows build (still useful) 
test-network.bat             # Multi-client test (functional)
```

## 🔄 **Migration Commands**

### Old → New Command Mapping

| Old Command | New Command |
|-------------|-------------|
| `genesis.bat` | `python paradigm-network.py start --genesis` |
| `launch-genesis.bat` | `python paradigm-network.py start --genesis` |
| `build-genesis-only.bat` | `python paradigm-network.py build` |
| `test-local-network.bat` | `python paradigm-network.py test --nodes 3` |
| `contributor-connect.bat` | `python paradigm-network.py start --peers IP:8080` |
| `setup-release.bat` | `python paradigm-network.py build --advanced` |

## 🚀 **New Unified Interface**

All network operations now use the modern Python launcher:

```bash
# Build system
python paradigm-network.py build [--advanced]

# Start genesis network
python paradigm-network.py start --genesis

# Join existing network  
python paradigm-network.py start --peers "192.168.1.100:8080"

# Multi-node testing
python paradigm-network.py test --nodes 5

# Wallet operations
python paradigm-network.py wallet create my_wallet
python paradigm-network.py wallet balance PAR1...
python paradigm-network.py wallet send PAR1... PAR1... 0.1

# Network status
python paradigm-network.py status
```

## 📁 **Documentation Files to Update**

### Files that reference removed scripts:
- `README.md` ✅ **(Updated)**
- `docs/QUICKSTART.md` ⚠️ **Needs update** 
- `docs/START-NETWORK.md` ⚠️ **Needs update**
- `docs/NETWORK_SETUP_GUIDE.md` ⚠️ **Needs update**

## 🗑️ **Safe Removal Commands**

**Windows:**
```batch
REM Remove outdated batch files
del genesis.bat
del launch-genesis.bat  
del build-genesis-only.bat
del test-local-network.bat
del contributor-connect.bat
del setup-release.bat

REM Remove any genesis-init scripts in target directories
del target\debug\genesis-init.bat 2>nul
del target\release\genesis-init.bat 2>nul
del target\paradigm-release\genesis-chain.bat 2>nul
```

**Linux/macOS:**
```bash
# Remove outdated batch files
rm -f genesis.bat launch-genesis.bat build-genesis-only.bat
rm -f test-local-network.bat contributor-connect.bat setup-release.bat

# Remove any genesis-init scripts in target directories  
rm -f target/debug/genesis-init.bat target/release/genesis-init.bat
rm -f target/paradigm-release/genesis-chain.bat
```

## ✨ **Benefits of the New System**

1. **Cross-Platform**: Works on Windows, Linux, and macOS
2. **Unified Interface**: Single script for all operations
3. **AI-Aware**: Understands new AI governance features
4. **Security-Enhanced**: Supports TLS/mTLS and HSM configuration
5. **Modern**: Python-based with proper error handling and progress feedback
6. **Maintainable**: Easy to extend with new features

## 🔧 **Migration Checklist**

- [ ] Test `paradigm-network.py` functionality
- [ ] Update documentation references
- [ ] Remove outdated batch files
- [ ] Update CI/CD scripts if any
- [ ] Inform team of new command structure
- [ ] Archive old scripts if needed for reference

---

**Next Steps**: Once testing is complete, run the removal commands to clean up the repository.