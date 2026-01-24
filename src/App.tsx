import { useState, useEffect } from 'react';
import { ImmutableEncryption, EncryptedFrame, VideoFrame } from './crypto/ImmutableEncryption';
import { Terminal } from './components/Terminal';
import { EncryptionDisplay } from './components/EncryptionDisplay';
import { VerificationDisplay } from './components/VerificationDisplay';
import { theme } from './theme';

type View = 'terminal' | 'encryption' | 'verification' | 'blockchain';

export default function App() {
  const [view, setView] = useState<View>('terminal');
  const [encryption] = useState(() => new ImmutableEncryption());
  const [frames, setFrames] = useState<EncryptedFrame[]>([]);
  const [terminalLog, setTerminalLog] = useState<string[]>([]);

  useEffect(() => {
    const initLog = [
      '╔══════════════════════════════════════╗',
      '║   REAL-TIME IMMUTABLE ENCRYPTIONS    ║',
      '║         Quantum-Secure System         ║',
      '║          Zero Dependencies            ║',
      '╚══════════════════════════════════════╝',
      '',
      '> System initialized with Web Crypto API',
      '> Quantum-resistant cryptography enabled',
      '> Blockchain anchoring system online',
      '> Ready for secure operations...',
      ''
    ];
    setTerminalLog(initLog);
  }, []);

  const addLog = (message: string) => {
    setTerminalLog(prev => [...prev, `> ${message}`]);
  };

  const handleEncryptFrame = async (imageData: string) => {
    try {
      addLog('Starting frame encryption...');
      
      const frame: VideoFrame = {
        timestamp: Date.now(),
        sequence: frames.length + 1,
        data: imageData,
        metadata: {
          deviceId: `device_${Math.random().toString(36).substr(2, 9)}`,
          location: { lat: 40.7128 + Math.random() * 0.01, lng: -74.0060 + Math.random() * 0.01 },
          resolution: { width: 1920, height: 1080 },
          fps: 30,
          codec: 'H.265'
        }
      };

      const encrypted = await encryption.encryptFrame(frame);
      setFrames(prev => [...prev, encrypted]);
      addLog(`Frame ${encrypted.sequence} encrypted successfully`);
      addLog(`Hash: ${encrypted.hash.substring(0, 16)}...`);
      addLog(`Blockchain anchor: ${encrypted.blockchainAnchors[0].transactionHash.substring(0, 16)}...`);
    } catch (error) {
      addLog(`ERROR: ${error}`);
    }
  };

  const handleVerifyFrames = async () => {
    try {
      addLog('Starting integrity verification...');
      const result = await encryption.verifyIntegrity(frames);
      addLog(`Verification completed: ${result.isValid ? 'VALID' : 'TAMPERED'}`);
      addLog(`Frames verified: ${result.frameCount}`);
      if (result.tamperEvidence) {
        addLog(`Tamper evidence: ${result.tamperEvidence}`);
      }
      addLog(`Court report generated: ${result.courtReport.evidenceId}`);
    } catch (error) {
      addLog(`ERROR: ${error}`);
    }
  };

  const Navigation = () => (
    <div style={{
      display: 'flex',
      gap: '2px',
      marginBottom: '20px',
      borderBottom: `1px solid ${theme.colors.primary}`,
      paddingBottom: '10px',
    }}>
      {(['terminal', 'encryption', 'verification', 'blockchain'] as View[]).map((v) => (
        <button
          key={v}
          onClick={() => setView(v)}
          style={{
            backgroundColor: view === v ? theme.colors.primary : 'transparent',
            color: theme.colors.text,
            border: `1px solid ${theme.colors.primary}`,
            padding: '8px 16px',
            fontFamily: 'monospace',
            fontSize: '12px',
            cursor: 'pointer',
            textTransform: 'uppercase',
            transition: 'all 0.2s',
          }}
          onMouseEnter={(e) => {
            if (view !== v) {
              e.currentTarget.style.backgroundColor = `${theme.colors.primary}33`;
            }
          }}
          onMouseLeave={(e) => {
            if (view !== v) {
              e.currentTarget.style.backgroundColor = 'transparent';
            }
          }}
        >
          {v}
        </button>
      ))}
    </div>
  );

  const renderView = () => {
    switch (view) {
      case 'terminal':
        return <Terminal log={terminalLog} />;
      case 'encryption':
        return <EncryptionDisplay onEncryptFrame={handleEncryptFrame} />;
      case 'verification':
        return <VerificationDisplay 
          frames={frames} 
          onVerify={handleVerifyFrames}
        />;
      case 'blockchain':
        return <div style={{ color: theme.colors.text }}>
          <h3>Blockchain Anchors</h3>
          <p>Total Anchored Frames: {frames.length}</p>
          {frames.map(frame => (
            <div key={frame.sequence} style={{ marginBottom: '10px' }}>
              <strong>Frame {frame.sequence}:</strong>
              <div style={{ fontSize: '11px', opacity: 0.8 }}>
                Tx: {frame.blockchainAnchors[0]?.transactionHash}
              </div>
            </div>
          ))}
        </div>;
      default:
        return <Terminal log={terminalLog} />;
    }
  };

  return (
    <div style={{
      height: '100vh',
      width: '100vw',
      backgroundColor: theme.colors.background,
      color: theme.colors.text,
      fontFamily: 'monospace',
      overflow: 'hidden',
      position: 'relative',
    }}>
      <div className="matrix-bg" style={{
        position: 'absolute',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        opacity: 0.1,
        zIndex: 0,
      }} />
      
      <div style={{
        position: 'relative',
        zIndex: 1,
        height: '100vh',
        display: 'flex',
        flexDirection: 'column',
        padding: '20px',
      }}>
        <div style={{
          textAlign: 'center',
          marginBottom: '20px',
        }}>
          <h1 style={{
            color: theme.colors.primary,
            fontSize: '20px',
            margin: 0,
            textShadow: `0 0 10px ${theme.colors.primary}`,
            letterSpacing: '2px',
          }}>
            QUANTUM-SAFE ENCRYPTION SYSTEM
          </h1>
          <div style={{
            fontSize: '12px',
            opacity: 0.7,
            marginTop: '5px',
          }}>
            Real-Time Immutable Encryptions • Zero Dependencies
          </div>
        </div>

        <Navigation />
        
        <div style={{
          flex: 1,
          overflow: 'auto',
          backgroundColor: 'rgba(0,0,0,0.8)',
          border: `1px solid ${theme.colors.primary}`,
          borderRadius: '8px',
          padding: '20px',
        }}>
          {renderView()}
        </div>

        <div style={{
          display: 'flex',
          justifyContent: 'space-between',
          fontSize: '10px',
          opacity: 0.7,
          marginTop: '10px',
        }}>
          <div>Status: ONLINE</div>
          <div>Frames: {frames.length}</div>
          <div>Crypto: AES-256-GCM</div>
          <div>Blockchain: SIMULATED</div>
        </div>
      </div>
    </div>
  );
}