# Immutable Encryption - Python API

Advanced AI-powered video evidence analysis for court-admissible, immutable evidence.

## Features

- **Real-time AI Analysis**: Object detection, face recognition, quality assessment
- **Court-Ready Reports**: Legal compliance and expert testimony support
- **Multi-Modal Processing**: Video, audio, and metadata analysis
- **Forensic Enhancement**: Environmental assessment and camera specifications
- **RESTful API**: Easy integration with evidence management systems

## Installation

```bash
pip install -r requirements.txt
```

## Quick Start

### Start the API Server

```bash
cd python_api
python main.py
```

The API will be available at `http://localhost:8000`

### Upload and Process Evidence

```python
import requests

# Upload video file
with open('evidence.mp4', 'rb') as f:
    response = requests.post(
        'http://localhost:8000/api/v1/evidence/upload',
        files={'file': f},
        data={
            'device_id': 'drone_001',
            'evidence_type': 'video',
            'location': '40.7128,-74.0060'
        },
        headers={'Authorization': 'Bearer demo-token'}
    )

evidence_id = response.json()['evidence_id']
print(f"Evidence uploaded with ID: {evidence_id}")

# Check processing progress
progress = requests.get(
    f'http://localhost:8000/api/v1/evidence/{evidence_id}/progress',
    headers={'Authorization': 'Bearer demo-token'}
)

print(f"Processing status: {progress.json()['status']}")
```

### Generate Court Report

```python
# Generate court-admissible report
report = requests.post(
    f'http://localhost:8000/api/v1/evidence/{evidence_id}/court-report',
    json={
        'report_type': 'full',
        'jurisdiction': 'US'
    },
    headers={'Authorization': 'Bearer demo-token'}
)

print(json.dumps(report.json(), indent=2))
```

## API Documentation

### Authentication

All API endpoints require Bearer token authentication. For demo purposes, use:

```
Authorization: Bearer demo-token
```

### Endpoints

#### Upload Evidence
```
POST /api/v1/evidence/upload
Content-Type: multipart/form-data
```

**Parameters:**
- `file`: Video file (required)
- `device_id`: Device identifier (optional)
- `evidence_type`: Type of evidence (optional)
- `location`: GPS coordinates (optional)

#### Get Processing Progress
```
GET /api/v1/evidence/{evidence_id}/progress
```

Returns current processing status and progress percentage.

#### Get Analysis Results
```
GET /api/v1/evidence/{evidence_id}/results
```

Returns complete AI analysis results including:
- Object detection summary
- Face recognition results
- Quality assessment
- Environmental analysis

#### Verify Evidence
```
POST /api/v1/evidence/{evidence_id}/verify
```

**Request Body:**
```json
{
  "verification_level": "forensic",
  "include_ai_analysis": true
}
```

#### Generate Court Report
```
POST /api/v1/evidence/{evidence_id}/court-report
```

**Request Body:**
```json
{
  "report_type": "full",
  "jurisdiction": "US"
}
```

### AI Analysis Features

#### Object Detection
- **Model**: YOLOv5
- **Classes**: 80 COCO object categories
- **Confidence Threshold**: 0.5
- **Output**: Bounding boxes, labels, confidence scores

#### Face Recognition
- **Detection**: Face locations in video frames
- **Recognition**: 128-dimension face embeddings
- **Features**: Landmarks, quality assessment

#### Quality Assessment
- **Metrics**: Sharpness, brightness, contrast, noise
- **Scoring**: 0.0 to 1.0 quality score
- **Factors**: Camera shake, lighting conditions

#### Environmental Analysis
- **Lighting**: Dark, normal, bright classification
- **Weather**: Sunny, cloudy, foggy estimation
- **Time**: Day/night/morning/evening detection

## Legal Compliance

The system adheres to:

- **ISO/IEC 27037:2012**: Guidelines for identification, collection, acquisition, and preservation of digital evidence
- **NIST SP 800-101**: Guidelines on mobile device forensics
- **Daubert Standard**: Scientific evidence admissibility criteria
- **FRE 901(b)**: Federal Rules of Evidence - authentication requirement

### Court Report Sections

1. **Chain of Custody**: Complete evidence timeline
2. **Technical Specifications**: Video properties and processing parameters
3. **AI Analysis Results**: Object, face, and quality analysis
4. **Legal Compliance**: Standards and jurisdiction requirements
5. **Blockchain Verification**: Cryptographic proof anchoring
6. **Expert Testimony Support**: Methodology and accuracy metrics

## Integration Examples

### Python Client

```python
class EvidenceClient:
    def __init__(self, base_url, token="demo-token"):
        self.base_url = base_url
        self.headers = {"Authorization": f"Bearer {token}"}
    
    async def upload_evidence(self, file_path, device_id="unknown"):
        with open(file_path, 'rb') as f:
            files = {'file': f}
            data = {'device_id': device_id}
            response = requests.post(
                f'{self.base_url}/api/v1/evidence/upload',
                files=files,
                data=data,
                headers=self.headers
            )
        return response.json()
    
    async def wait_for_processing(self, evidence_id, timeout=300):
        import time
        start_time = time.time()
        
        while time.time() - start_time < timeout:
            progress = requests.get(
                f'{self.base_url}/api/v1/evidence/{evidence_id}/progress',
                headers=self.headers
            ).json()
            
            if progress['status'] == 'completed':
                return requests.get(
                    f'{self.base_url}/api/v1/evidence/{evidence_id}/results',
                    headers=self.headers
                ).json()
            elif progress['status'] == 'failed':
                raise Exception(f"Processing failed: {progress['error_message']}")
            
            await asyncio.sleep(2)
        
        raise TimeoutError("Processing timeout")

# Usage
client = EvidenceClient("http://localhost:8000")
evidence = await client.upload_evidence("surveillance_footage.mp4", "camera_01")
results = await client.wait_for_processing(evidence['evidence_id'])
```

### JavaScript Client

```javascript
class EvidenceClient {
    constructor(baseUrl, token = 'demo-token') {
        this.baseUrl = baseUrl;
        this.headers = {
            'Authorization': `Bearer ${token}`
        };
    }

    async uploadEvidence(file, deviceId = 'unknown') {
        const formData = new FormData();
        formData.append('file', file);
        formData.append('device_id', deviceId);

        const response = await fetch(`${this.baseUrl}/api/v1/evidence/upload`, {
            method: 'POST',
            body: formData,
            headers: this.headers
        });

        return await response.json();
    }

    async getResults(evidenceId) {
        const response = await fetch(
            `${this.baseUrl}/api/v1/evidence/${evidenceId}/results`,
            { headers: this.headers }
        );
        return await response.json();
    }

    async generateCourtReport(evidenceId, reportType = 'full') {
        const response = await fetch(
            `${this.baseUrl}/api/v1/evidence/${evidenceId}/court-report`,
            {
                method: 'POST',
                headers: {
                    ...this.headers,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    report_type: reportType,
                    jurisdiction: 'US'
                })
            }
        );
        return await response.json();
    }
}

// Usage
const client = new EvidenceClient('http://localhost:8000');

// Upload file from form
document.getElementById('upload-form').addEventListener('submit', async (e) => {
    e.preventDefault();
    const file = document.getElementById('file-input').files[0];
    
    try {
        const evidence = await client.uploadEvidence(file, 'camera_01');
        console.log('Evidence uploaded:', evidence.evidence_id);
        
        // Check progress periodically
        const checkProgress = async () => {
            const progress = await fetch(
                `${client.baseUrl}/api/v1/evidence/${evidence.evidence_id}/progress`,
                { headers: client.headers }
            ).then(r => r.json());
            
            console.log('Progress:', progress.status, `${(progress.progress * 100).toFixed(1)}%`);
            
            if (progress.status === 'completed') {
                const results = await client.getResults(evidence.evidence_id);
                console.log('Analysis complete:', results);
            } else if (progress.status === 'failed') {
                console.error('Processing failed:', progress.error_message);
            } else {
                setTimeout(checkProgress, 2000);
            }
        };
        
        checkProgress();
    } catch (error) {
        console.error('Upload failed:', error);
    }
});
```

## Performance

- **Processing Time**: 2-5 minutes for 10-minute video (sampling 1 frame per second)
- **Memory Usage**: ~2GB for 4K video processing
- **GPU Acceleration**: Automatic CUDA detection and usage
- **Throughput**: Up to 10 concurrent video processing jobs

## Deployment

### Docker

```dockerfile
FROM python:3.9-slim

WORKDIR /app
COPY requirements.txt .
RUN pip install -r requirements.txt

COPY . .
EXPOSE 8000

CMD ["uvicorn", "main:app", "--host", "0.0.0.0", "--port", "8000"]
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: evidence-ai-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: evidence-ai-api
  template:
    metadata:
      labels:
        app: evidence-ai-api
    spec:
      containers:
      - name: api
        image: evidence-ai-api:latest
        ports:
        - containerPort: 8000
        resources:
          requests:
            memory: "2Gi"
            cpu: "1000m"
          limits:
            memory: "4Gi"
            cpu: "2000m"
        env:
        - name: CUDA_VISIBLE_DEVICES
          value: "0"
```

## Security

- **Authentication**: Bearer token-based (JWT in production)
- **Encryption**: All data encrypted at rest and in transit
- **Audit Trail**: Complete logging of all operations
- **Rate Limiting**: API endpoint protection
- **Input Validation**: Comprehensive input sanitization

## Monitoring

- **Health Checks**: `/health` endpoint
- **Metrics**: Processing time, success rates, system load
- **Logging**: Structured JSON logs with correlation IDs
- **Alerts**: Failed processing, high error rates

## Support

For technical support or questions:
- Check the API documentation at `/docs`
- Review the log files for error details
- Contact the development team for assistance