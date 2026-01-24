import React, { useRef, useState } from 'react';
import { theme } from '../theme';

interface EncryptionDisplayProps {
  onEncryptFrame: (imageData: string) => void;
}

export function EncryptionDisplay({ onEncryptFrame }: EncryptionDisplayProps) {
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [isProcessing, setIsProcessing] = useState(false);

  const handleFileSelect = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;

    setIsProcessing(true);
    
    try {
      const reader = new FileReader();
      reader.onload = (e) => {
        const imageData = e.target?.result as string;
        onEncryptFrame(imageData);
      };
      reader.readAsDataURL(file);
    } catch (error) {
      console.error('Error processing file:', error);
    } finally {
      setIsProcessing(false);
    }
  };

  const handleCameraCapture = async () => {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ 
        video: { width: 1920, height: 1080 } 
      });
      
      const video = document.createElement('video');
      video.srcObject = stream;
      video.play();

      // Capture frame after a short delay
      setTimeout(() => {
        const canvas = document.createElement('canvas');
        canvas.width = video.videoWidth;
        canvas.height = video.videoHeight;
        
        const ctx = canvas.getContext('2d');
        if (ctx) {
          ctx.drawImage(video, 0, 0);
          const imageData = canvas.toDataURL('image/jpeg');
          onEncryptFrame(imageData);
        }
        
        // Stop camera
        stream.getTracks().forEach(track => track.stop());
      }, 1000);
    } catch (error) {
      console.error('Error accessing camera:', error);
    }
  };

  const generateSampleData = () => {
    // Generate a simple test pattern
    const canvas = document.createElement('canvas');
    canvas.width = 1920;
    canvas.height = 1080;
    
    const ctx = canvas.getContext('2d');
    if (ctx) {
      // Create a test pattern with timestamp
      ctx.fillStyle = '#000';
      ctx.fillRect(0, 0, canvas.width, canvas.height);
      
      // Add timestamp
      ctx.fillStyle = '#00ff41';
      ctx.font = '48px monospace';
      ctx.fillText(`QUANTUM ENCRYPTION TEST`, 50, 100);
      ctx.fillText(`Timestamp: ${Date.now()}`, 50, 200);
      ctx.fillText(`Frame ID: ${Math.random().toString(36).substr(2, 9)}`, 50, 300);
      
      // Add encryption pattern
      for (let i = 0; i < 50; i++) {
        ctx.fillStyle = `rgba(0, 255, 65, ${Math.random()})`;
        ctx.fillRect(
          Math.random() * canvas.width,
          Math.random() * canvas.height,
          Math.random() * 100 + 20,
          Math.random() * 100 + 20
        );
      }
      
      const imageData = canvas.toDataURL('image/jpeg');
      onEncryptFrame(imageData);
    }
  };

  return (
    <div style={{
      height: '100%',
      display: 'flex',
      flexDirection: 'column',
      gap: theme.spacing.lg,
    }}>
      <div>
        <h3 style={{
          color: theme.colors.primary,
          margin: 0,
          marginBottom: theme.spacing.md,
        }}>
          FRAME ENCRYPTION MODULE
        </h3>
        <p style={{
          color: theme.colors.muted,
          fontSize: '12px',
          margin: 0,
        }}>
          Encrypt video frames with quantum-safe algorithms and blockchain anchoring
        </p>
      </div>

      <div style={{
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(250px, 1fr))',
        gap: theme.spacing.md,
      }}>
        {/* File Upload */}
        <div style={{
          border: `2px dashed ${theme.colors.primary}`,
          borderRadius: theme.borderRadius.md,
          padding: theme.spacing.lg,
          textAlign: 'center',
          cursor: 'pointer',
          transition: 'all 0.2s',
        }}
        onClick={() => fileInputRef.current?.click()}
        onMouseEnter={(e) => {
          e.currentTarget.style.backgroundColor = `${theme.colors.primary}11`;
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.backgroundColor = 'transparent';
        }}
        >
          <div style={{ fontSize: '24px', marginBottom: theme.spacing.sm }}>📁</div>
          <div style={{ color: theme.colors.text, marginBottom: theme.spacing.sm }}>
            Upload Image/Video
          </div>
          <div style={{ fontSize: '10px', color: theme.colors.muted }}>
            JPG, PNG, MP4 supported
          </div>
          <input
            ref={fileInputRef}
            type="file"
            accept="image/*,video/*"
            onChange={handleFileSelect}
            style={{ display: 'none' }}
          />
        </div>

        {/* Camera Capture */}
        <div style={{
          border: `2px solid ${theme.colors.primary}`,
          borderRadius: theme.borderRadius.md,
          padding: theme.spacing.lg,
          textAlign: 'center',
          cursor: 'pointer',
          transition: 'all 0.2s',
        }}
        onClick={handleCameraCapture}
        onMouseEnter={(e) => {
          e.currentTarget.style.backgroundColor = `${theme.colors.primary}11`;
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.backgroundColor = 'transparent';
        }}
        >
          <div style={{ fontSize: '24px', marginBottom: theme.spacing.sm }}>📸</div>
          <div style={{ color: theme.colors.text, marginBottom: theme.spacing.sm }}>
            Camera Capture
          </div>
          <div style={{ fontSize: '10px', color: theme.colors.muted }}>
            Real-time frame encryption
          </div>
        </div>

        {/* Generate Sample */}
        <div style={{
          border: `2px solid ${theme.colors.secondary}`,
          borderRadius: theme.borderRadius.md,
          padding: theme.spacing.lg,
          textAlign: 'center',
          cursor: 'pointer',
          transition: 'all 0.2s',
        }}
        onClick={generateSampleData}
        onMouseEnter={(e) => {
          e.currentTarget.style.backgroundColor = `${theme.colors.secondary}11`;
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.backgroundColor = 'transparent';
        }}
        >
          <div style={{ fontSize: '24px', marginBottom: theme.spacing.sm }}>⚛️</div>
          <div style={{ color: theme.colors.text, marginBottom: theme.spacing.sm }}>
            Generate Test Pattern
          </div>
          <div style={{ fontSize: '10px', color: theme.colors.muted }}>
            Quantum encryption test
          </div>
        </div>
      </div>

      {/* Processing Status */}
      {isProcessing && (
        <div style={{
          backgroundColor: theme.colors.surface,
          border: `1px solid ${theme.colors.warning}`,
          borderRadius: theme.borderRadius.md,
          padding: theme.spacing.md,
          textAlign: 'center',
        }}>
          <div style={{ color: theme.colors.warning }}>
            ⚡ Processing frame... Quantum encryption in progress
          </div>
        </div>
      )}

      {/* Encryption Info */}
      <div style={{
        backgroundColor: theme.colors.surface,
        border: `1px solid ${theme.colors.primary}`,
        borderRadius: theme.borderRadius.md,
        padding: theme.spacing.lg,
        fontSize: '11px',
        lineHeight: '1.6',
      }}>
        <div style={{ marginBottom: theme.spacing.md, color: theme.colors.primary }}>
          ENCRYPTION SPECIFICATIONS:
        </div>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '10px' }}>
          <div>🔐 Algorithm: AES-256-GCM</div>
          <div>🧮 Key Size: 256 bits</div>
          <div>⛓️ Hash: SHA-256</div>
          <div>🎯 Authentication: HMAC-SHA256</div>
          <div>📦 IV Size: 96 bits</div>
          <div>🔗 Blockchain: Simulated PoW</div>
          <div>⚡ Quantum: Post-Quantum Ready</div>
          <div>🛡️ Zero Dependencies</div>
        </div>
      </div>
    </div>
  );
}