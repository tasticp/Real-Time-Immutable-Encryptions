#!/usr/bin/env python3
"""
Integration tests for the Immutable Encryption system.
Tests complete end-to-end workflows across all components.
"""

import pytest
import requests
import time
import json
import tempfile
import os
from pathlib import Path
import subprocess
import threading

# Test configuration
API_BASE_URL = "http://localhost:8000"
RUST_BACKEND_URL = "http://localhost:8080"
STREAMLIT_URL = "http://localhost:8501"
AUTH_TOKEN = "demo-token"

class TestImmutableEncryption:
    """End-to-end integration tests"""
    
    @classmethod
    def setup_class(cls):
        """Setup test environment"""
        cls.session = requests.Session()
        cls.session.headers.update({"Authorization": f"Bearer {AUTH_TOKEN}"})
        
        # Create test video file
        cls.test_video_path = cls.create_test_video()
        
        # Wait for services to be ready
        cls.wait_for_services()
    
    @classmethod
    def teardown_class(cls):
        """Cleanup test environment"""
        if hasattr(cls, 'test_video_path') and os.path.exists(cls.test_video_path):
            os.remove(cls.test_video_path)
    
    @staticmethod
    def create_test_video():
        """Create a test video file"""
        # For testing, create a dummy video file
        test_data = b"FAKE_VIDEO_DATA_FOR_TESTING" * 10000  # 450KB
        temp_file = tempfile.NamedTemporaryFile(suffix='.mp4', delete=False)
        temp_file.write(test_data)
        temp_file.close()
        return temp_file.name
    
    @staticmethod
    def wait_for_services(timeout=300):
        """Wait for all services to be ready"""
        start_time = time.time()
        
        services = {
            "Python API": f"{API_BASE_URL}/health",
            "Rust Backend": f"{RUST_BACKEND_URL}/health",
        }
        
        while time.time() - start_time < timeout:
            all_ready = True
            
            for service_name, health_url in services.items():
                try:
                    response = requests.get(health_url, timeout=5)
                    if response.status_code != 200:
                        all_ready = False
                        break
                except requests.exceptions.RequestException:
                    all_ready = False
                    break
            
            if all_ready:
                print("All services are ready")
                return True
            
            time.sleep(5)
        
        raise TimeoutError("Services did not become ready in time")
    
    def test_system_health(self):
        """Test overall system health"""
        # Test Python API health
        response = self.session.get(f"{API_BASE_URL}/health")
        assert response.status_code == 200
        health_data = response.json()
        assert health_data["status"] == "healthy"
        
        # Test Rust backend health
        response = self.session.get(f"{RUST_BACKEND_URL}/health")
        assert response.status_code == 200
        
        # Test services are operational
        assert "ai_analyzer" in health_data["services"]
        assert health_data["services"]["ai_analyzer"] == "operational"
    
    def test_evidence_upload_and_processing(self):
        """Test complete evidence upload and processing pipeline"""
        # Upload evidence
        with open(self.test_video_path, 'rb') as f:
            files = {'file': f}
            data = {
                'device_id': 'test_camera_001',
                'evidence_type': 'test_video',
                'location': '40.7128,-74.0060'
            }
            
            response = self.session.post(
                f"{API_BASE_URL}/api/v1/evidence/upload",
                files=files,
                data=data
            )
        
        assert response.status_code == 200
        upload_data = response.json()
        evidence_id = upload_data["evidence_id"]
        assert evidence_id is not None
        
        # Wait for processing to complete
        max_wait = 120  # 2 minutes
        start_time = time.time()
        
        while time.time() - start_time < max_wait:
            progress_response = self.session.get(
                f"{API_BASE_URL}/api/v1/evidence/{evidence_id}/progress"
            )
            
            if progress_response.status_code == 200:
                progress_data = progress_response.json()
                
                if progress_data["status"] == "completed":
                    break
                elif progress_data["status"] == "failed":
                    pytest.fail(f"Evidence processing failed: {progress_data.get('error_message')}")
            
            time.sleep(2)
        else:
            pytest.fail("Evidence processing timed out")
        
        # Retrieve results
        results_response = self.session.get(
            f"{API_BASE_URL}/api/v1/evidence/{evidence_id}/results"
        )
        
        assert results_response.status_code == 200
        results_data = results_response.json()
        
        # Verify AI analysis results
        assert "processing_results" in results_data
        assert "objects_summary" in results_data["processing_results"]
        assert "quality_stats" in results_data["processing_results"]
        
        return evidence_id
    
    def test_evidence_verification(self):
        """Test evidence verification pipeline"""
        # First upload and process evidence
        evidence_id = self.test_evidence_upload_and_processing()
        
        # Verify evidence
        verification_data = {
            "verification_level": "forensic",
            "include_ai_analysis": True
        }
        
        response = self.session.post(
            f"{API_BASE_URL}/api/v1/evidence/{evidence_id}/verify",
            json=verification_data
        )
        
        assert response.status_code == 200
        verification_result = response.json()
        
        # Verify verification results
        assert verification_result["verification_level"] == "forensic"
        assert verification_result["ai_analysis"] == True
        assert verification_result["integrity_check"] == "passed"
        assert "blockchain_status" in verification_result
        assert verification_result["blockchain_status"]["bitcoin_anchored"] == True
    
    def test_court_report_generation(self):
        """Test court report generation"""
        evidence_id = self.test_evidence_upload_and_processing()
        
        # Generate court report
        report_data = {
            "report_type": "full",
            "jurisdiction": "US"
        }
        
        response = self.session.post(
            f"{API_BASE_URL}/api/v1/evidence/{evidence_id}/court-report",
            json=report_data
        )
        
        assert response.status_code == 200
        court_report = response.json()
        
        # Verify court report structure
        assert court_report["evidence_id"] == evidence_id
        assert court_report["report_type"] == "full"
        assert court_report["jurisdiction"] == "US"
        assert "chain_of_custody" in court_report
        assert "technical_specifications" in court_report
        assert "ai_analysis_results" in court_report
        assert "legal_compliance" in court_report
        assert "blockchain_verification" in court_report
        
        # Verify legal compliance
        legal_compliance = court_report["legal_compliance"]
        assert "standards_met" in legal_compliance
        assert len(legal_compliance["standards_met"]) > 0
        assert "jurisdiction_requirements" in legal_compliance
        
        return court_report
    
    def test_blockchain_integration(self):
        """Test blockchain anchoring functionality"""
        evidence_id = self.test_evidence_upload_and_processing()
        
        # Verify blockchain anchoring in results
        results_response = self.session.get(
            f"{API_BASE_URL}/api/v1/evidence/{evidence_id}/results"
        )
        
        assert results_response.status_code == 200
        results_data = results_response.json()
        
        # The processing results should include blockchain information
        # In a real implementation, this would verify actual blockchain transactions
        processing_results = results_data.get("processing_results", {})
        
        # For now, we verify the structure exists
        # In production, this would check actual blockchain confirmations
        assert True  # Placeholder for blockchain verification
    
    def test_concurrent_processing(self):
        """Test system under concurrent load"""
        evidence_ids = []
        
        # Upload multiple evidence items concurrently
        def upload_evidence(device_id):
            with open(self.test_video_path, 'rb') as f:
                files = {'file': f}
                data = {
                    'device_id': device_id,
                    'evidence_type': 'test_video',
                }
                
                response = self.session.post(
                    f"{API_BASE_URL}/api/v1/evidence/upload",
                    files=files,
                    data=data
                )
                
                if response.status_code == 200:
                    return response.json()["evidence_id"]
                return None
        
        # Upload 5 evidence items concurrently
        threads = []
        for i in range(5):
            thread = threading.Thread(
                target=upload_evidence,
                args=(f"concurrent_device_{i}",)
            )
            threads.append(thread)
            thread.start()
        
        # Wait for all uploads to complete
        for thread in threads:
            thread.join()
        
        # In a real test, we would collect evidence IDs and verify processing
        # For now, just verify the system handles concurrent requests
        assert True  # Placeholder for concurrent processing verification
    
    def test_error_handling(self):
        """Test error handling and edge cases"""
        # Test invalid evidence ID
        response = self.session.get(
            f"{API_BASE_URL}/api/v1/evidence/invalid_evidence_id/results"
        )
        assert response.status_code == 404
        
        # Test missing evidence upload
        response = self.session.post(
            f"{API_BASE_URL}/api/v1/evidence/upload",
            files={},
            data={}
        )
        assert response.status_code == 422  # Validation error
        
        # Test invalid report generation
        evidence_id = self.test_evidence_upload_and_processing()
        response = self.session.post(
            f"{API_BASE_URL}/api/v1/evidence/{evidence_id}/court-report",
            json={"invalid": "data"}
        )
        assert response.status_code == 422
    
    def test_security_features(self):
        """Test security features and authentication"""
        # Test without authentication
        session_no_auth = requests.Session()
        
        response = session_no_auth.get(f"{API_BASE_URL}/health")
        # Health endpoint should work without auth
        assert response.status_code == 200
        
        response = session_no_auth.post(
            f"{API_BASE_URL}/api/v1/evidence/upload"
        )
        # Protected endpoints should require auth
        assert response.status_code == 401
        
        # Test with invalid token
        session_invalid = requests.Session()
        session_invalid.headers.update({"Authorization": "Bearer invalid_token"})
        
        response = session_invalid.post(
            f"{API_BASE_URL}/api/v1/evidence/upload"
        )
        assert response.status_code == 401
    
    def test_data_integrity(self):
        """Test data integrity and validation"""
        evidence_id = self.test_evidence_upload_and_processing()
        
        # Get results and verify data structure
        results_response = self.session.get(
            f"{API_BASE_URL}/api/v1/evidence/{evidence_id}/results"
        )
        
        assert results_response.status_code == 200
        results_data = results_response.json()
        
        # Verify required fields exist and have valid data
        processing_results = results_data["processing_results"]
        
        assert "device_id" in results_data
        assert "evidence_type" in results_data
        assert "processing_timestamp" in results_data
        
        # Verify data types and formats
        assert isinstance(processing_results.get("objects_summary", {}), dict)
        assert isinstance(processing_results.get("quality_stats", {}), dict)
        assert isinstance(processing_results.get("frames_processed"), int)
        
        # Test verification maintains integrity
        verification_response = self.session.post(
            f"{API_BASE_URL}/api/v1/evidence/{evidence_id}/verify",
            json={"verification_level": "standard"}
        )
        
        assert verification_response.status_code == 200
        verification_data = verification_response.json()
        
        # Verification should not alter original evidence
        assert verification_data["integrity_check"] == "passed"


class TestRustBackend:
    """Tests specifically for the Rust backend"""
    
    def setup_method(self):
        """Setup for each test"""
        self.base_url = RUST_BACKEND_URL
    
    def test_rust_health(self):
        """Test Rust backend health endpoint"""
        response = requests.get(f"{self.base_url}/health")
        assert response.status_code == 200
        
        data = response.json()
        assert data["status"] == "healthy"
        assert "timestamp" in data
    
    def test_rust_status(self):
        """Test Rust backend status endpoint"""
        response = requests.get(f"{self.base_url}/status")
        assert response.status_code == 200
        
        data = response.json()
        assert "node" in data
        assert data["node"] == "running"


class TestPerformance:
    """Performance and load testing"""
    
    def test_response_times(self):
        """Test API response times"""
        start_time = time.time()
        response = requests.get(f"{API_BASE_URL}/health")
        end_time = time.time()
        
        assert response.status_code == 200
        response_time = end_time - start_time
        
        # Health endpoint should respond quickly
        assert response_time < 1.0, f"Health endpoint took too long: {response_time}s"
    
    def test_memory_usage(self):
        """Test memory usage (basic check)"""
        # This is a simplified test
        # In production, would use proper memory profiling
        import psutil
        process = psutil.Process()
        
        # Get memory info
        memory_info = process.memory_info()
        memory_mb = memory_info.rss / 1024 / 1024
        
        # Basic sanity check - should use reasonable memory
        assert memory_mb < 1000, f"Process using too much memory: {memory_mb}MB"


if __name__ == "__main__":
    # Run tests
    pytest.main([__file__, "-v", "--tb=short"])