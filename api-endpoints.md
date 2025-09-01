# 🔮 Paradigm SNT Network API Endpoints

## Network Overview
- `GET /api/network/stats` - Network statistics and health
- `GET /api/network/topology` - Network graph data for visualization
- `GET /api/network/events` - Recent network events

## Keeper Management  
- `GET /api/keepers/list` - List all active keepers
- `GET /api/keepers/{id}` - Get specific keeper details
- `POST /api/keepers/register` - Register new keeper
- `GET /api/keepers/{id}/snts` - Get keeper's SNT collection

## SNT System
- `GET /api/snt/overview` - SNT system statistics
- `GET /api/snt/types` - Available SNT types
- `GET /api/snt/list` - List all SNTs (paginated)
- `GET /api/snt/{id}` - Get specific SNT details
- `POST /api/snt/mint` - Mint new SNT (admin)
- `GET /api/snt/evolution/{id}` - Get SNT evolution history

## Storage Operations
- `POST /api/storage/store` - Store new sigil
- `GET /api/storage/retrieve/{hash}` - Retrieve sigil data
- `GET /api/storage/list` - List stored sigils
- `DELETE /api/storage/{hash}` - Remove sigil (if authorized)

## Fusion System
- `GET /api/fusion/templates` - Available fusion templates
- `POST /api/fusion/start` - Start fusion ritual
- `GET /api/fusion/{id}/preview` - Preview fusion result
- `POST /api/fusion/{id}/complete` - Complete fusion ritual
- `GET /api/fusion/history` - Fusion history

## Rewards & Economics
- `GET /api/rewards/pending` - Pending reward distributions  
- `GET /api/rewards/history` - Reward distribution history
- `GET /api/economics/overview` - Token economics overview
- `GET /api/economics/circulation` - Token circulation stats

## Real-time Data (WebSocket)
- `WS /ws/network` - Live network updates
- `WS /ws/snts` - Live SNT events (minting, evolution, etc)
- `WS /ws/fusion` - Live fusion ritual updates

## Example Requests

### Get Network Stats
```bash
curl http://localhost:8080/api/network/stats
```

### Register as Keeper
```bash  
curl -X POST http://localhost:8080/api/keepers/register \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Alice",
    "capacity": "10GB",
    "address": "127.0.0.1:9000"
  }'
```

### Store Data Sigil
```bash
curl -X POST http://localhost:8080/api/storage/store \
  -H "Content-Type: application/json" \
  -d '{
    "data": "SGVsbG8gU05UIFdvcmxkIQ==",
    "filename": "hello.txt",
    "category": "Document",
    "importance": "Standard"
  }'
```

### Start Fusion Ritual
```bash
curl -X POST http://localhost:8080/api/fusion/start \
  -H "Content-Type: application/json" \
  -d '{
    "sigils": ["hash1", "hash2"],
    "mode": "synthesis",
    "target_element": "aether"
  }'
```

### Get SNT Collection
```bash
curl http://localhost:8080/api/snt/list?holder=keeper_123
```

## Response Formats

All responses follow this structure:
```json
{
  "success": true,
  "data": { ... },
  "timestamp": 1703123456,
  "network_height": 12345
}
```

Error responses:
```json
{
  "success": false,
  "error": "Error description",
  "code": "ERROR_CODE",
  "timestamp": 1703123456
}
```