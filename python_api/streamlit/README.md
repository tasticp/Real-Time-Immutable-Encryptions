# Court Evidence Verification Portal

Streamlit-based web interface for the Immutable Encryption system's court-ready evidence verification.

## Features

- **Dashboard Overview**: Real-time system metrics and recent evidence
- **Evidence Upload**: Secure upload with progress tracking
- **Detailed Analysis**: AI-powered content analysis visualization
- **Court Reports**: Generate legally-compliant evidence reports
- **Blockchain Verification**: Multi-chain anchoring status and proofs
- **Legal Compliance**: Jurisdiction-specific requirements and standards

## Installation

```bash
pip install streamlit plotly pandas pillow requests
```

## Running the Application

```bash
cd python_api/streamlit
streamlit run app.py
```

The application will be available at `http://localhost:8501`

## Pages Overview

### 1. Dashboard
- System metrics and key performance indicators
- Recent evidence list with status indicators
- Blockchain anchoring status overview
- Real-time processing statistics

### 2. Upload Evidence
- Drag-and-drop file upload interface
- Metadata collection forms
- Real-time processing progress
- Upload confirmation and evidence ID generation

### 3. Evidence Details
- Comprehensive AI analysis results
- Object detection visualization
- Quality assessment metrics
- Frame-by-frame analysis timeline
- Face recognition results

### 4. Court Report
- Customizable report generation
- Multiple jurisdiction support
- Download in PDF, JSON, or text formats
- Executive summary creation

### 5. Blockchain Verification
- Multi-chain anchoring status
- Transaction confirmation details
- Cryptographic proof verification
- Hash chain visualization

### 6. Legal Compliance
- Standards compliance overview
- Jurisdiction-specific requirements
- Expert testimony preparation tools
- Audit trail documentation

## API Integration

The Streamlit app integrates with the FastAPI backend running on port 8000. Ensure both services are running:

```bash
# Terminal 1: Start FastAPI backend
cd python_api
python main.py

# Terminal 2: Start Streamlit frontend
cd python_api/streamlit  
streamlit run app.py
```

## Configuration

Environment variables:
- `API_BASE_URL`: Backend API URL (default: http://localhost:8000)
- `AUTH_TOKEN`: Authentication token (default: demo-token)

## Customization

### Adding New Pages

```python
def new_page():
    st.header("🆕 New Page")
    # Page content here

# Add to main() function:
if page == "New Page":
    new_page()
```

### Custom Metrics

```python
# Add new metric cards in dashboard()
st.markdown("""
<div class="metric-card">
    <h3>🆕 New Metric</h3>
    <h2>123</h2>
    <p>+5% this week</p>
</div>
""", unsafe_allow_html=True)
```

### Additional Charts

```python
import plotly.express as px

# Create new chart
df = pd.DataFrame({
    'Category': ['A', 'B', 'C'],
    'Value': [10, 20, 15]
})

fig = px.bar(df, x='Category', y='Value', title='New Chart')
st.plotly_chart(fig)
```

## Security Features

- Authentication via Bearer tokens
- Secure file upload with validation
- Encrypted communication with backend
- Session-based state management
- Input sanitization and validation

## Mobile Responsiveness

The interface is designed to work on:
- Desktop browsers (Chrome, Firefox, Safari, Edge)
- Tablet devices (iPad, Android tablets)
- Mobile phones (iOS, Android)

## Performance Optimization

- Lazy loading of large datasets
- Efficient chart rendering with Plotly
- Caching of API responses
- Optimized image processing
- Minimal data transfer

## Troubleshooting

### Common Issues

1. **API Connection Failed**
   - Ensure FastAPI backend is running on port 8000
   - Check firewall settings
   - Verify API_BASE_URL environment variable

2. **Upload Timeout**
   - Check file size limits
   - Verify sufficient disk space
   - Monitor backend logs

3. **Charts Not Displaying**
   - Ensure plotly is installed
   - Check browser console for JavaScript errors
   - Verify internet connectivity for external resources

### Debug Mode

Enable debug logging:

```python
import logging
logging.basicConfig(level=logging.DEBUG)
```

Add debugging information:

```python
st.write("Debug info:", variable)
```

## Browser Support

- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+

## Deployment

### Streamlit Cloud

Deploy to Streamlit Cloud by connecting your GitHub repository.

### Docker

```dockerfile
FROM python:3.9-slim

WORKDIR /app
COPY requirements.txt .
RUN pip install -r requirements.txt

COPY . .
EXPOSE 8501

CMD ["streamlit", "run", "app.py", "--server.port=8501", "--server.address=0.0.0.0"]
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: streamlit-ui
spec:
  replicas: 2
  selector:
    matchLabels:
      app: streamlit-ui
  template:
    metadata:
      labels:
        app: streamlit-ui
    spec:
      containers:
      - name: streamlit
        image: evidence-streamlit:latest
        ports:
        - containerPort: 8501
        env:
        - name: API_BASE_URL
          value: "http://fastapi-service:8000"
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.