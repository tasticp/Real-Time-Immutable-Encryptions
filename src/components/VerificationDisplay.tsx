
import { EncryptedFrame } from '../crypto/ImmutableEncryption';
import { theme } from '../theme';

interface VerificationDisplayProps {
  frames: EncryptedFrame[];
  onVerify: () => void;
}

export function VerificationDisplay({ frames, onVerify }: VerificationDisplayProps) {
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
          INTEGRITY VERIFICATION MODULE
        </h3>
        <p style={{
          color: theme.colors.muted,
          fontSize: '12px',
          margin: 0,
        }}>
          Verify cryptographic integrity and blockchain confirmations
        </p>
      </div>

      {/* Verification Controls */}
      <div style={{
        display: 'flex',
        gap: theme.spacing.md,
        alignItems: 'center',
      }}>
        <button
          onClick={onVerify}
          disabled={frames.length === 0}
          style={{
            backgroundColor: frames.length > 0 ? theme.colors.primary : theme.colors.muted,
            color: theme.colors.background,
            border: 'none',
            padding: `${theme.spacing.md}px ${theme.spacing.lg}px`,
            borderRadius: theme.borderRadius.md,
            fontFamily: 'monospace',
            fontSize: '14px',
            cursor: frames.length > 0 ? 'pointer' : 'not-allowed',
            transition: 'all 0.2s',
          }}
        >
          🔍 VERIFY ALL FRAMES
        </button>
        
        <div style={{
          color: theme.colors.muted,
          fontSize: '12px',
        }}>
          Frames available: {frames.length}
        </div>
      </div>

      {/* Frame List */}
      <div style={{
        flex: 1,
        overflow: 'auto',
        backgroundColor: theme.colors.surface,
        border: `1px solid ${theme.colors.primary}`,
        borderRadius: theme.borderRadius.md,
        padding: theme.spacing.md,
      }}>
        <div style={{
          color: theme.colors.primary,
          marginBottom: theme.spacing.md,
          fontSize: '14px',
          fontWeight: 'bold',
        }}>
          ENCRYPTED FRAMES:
        </div>
        
        {frames.length === 0 ? (
          <div style={{
            textAlign: 'center',
            color: theme.colors.muted,
            padding: theme.spacing.xl,
          }}>
            No frames encrypted yet. Go to Encryption module to start.
          </div>
        ) : (
          frames.map((frame) => (
            <div
              key={frame.sequence}
              style={{
                backgroundColor: theme.colors.background,
                border: `1px solid ${theme.colors.muted}`,
                borderRadius: theme.borderRadius.sm,
                padding: theme.spacing.md,
                marginBottom: theme.spacing.sm,
                fontSize: '11px',
                lineHeight: '1.8',
              }}
            >
              <div style={{
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
                marginBottom: theme.spacing.sm,
              }}>
                <span style={{ color: theme.colors.primary, fontWeight: 'bold' }}>
                  Frame #{frame.sequence}
                </span>
                <span style={{ color: theme.colors.muted, fontSize: '10px' }}>
                  {new Date(frame.timestamp).toLocaleString()}
                </span>
              </div>
              
              <div style={{ display: 'grid', gap: '4px' }}>
                <div><strong>Hash:</strong> {frame.hash.substring(0, 32)}...</div>
                <div><strong>Previous:</strong> {frame.previousHash.substring(0, 32)}...</div>
                <div><strong>Device:</strong> {frame.metadata.deviceId}</div>
                <div><strong>Resolution:</strong> {frame.metadata.resolution.width}x{frame.metadata.resolution.height}</div>
                <div><strong>Codec:</strong> {frame.metadata.codec}</div>
                
                {frame.blockchainAnchors.map((anchor, anchorIndex) => (
                  <div key={anchorIndex} style={{ marginTop: '4px' }}>
                    <strong>Blockchain:</strong> {anchor.chain} | 
                    <strong> Block:</strong> {anchor.blockNumber} | 
                    <strong> Confirmations:</strong> {anchorIndex + 1}/12
                  </div>
                ))}
              </div>
            </div>
          ))
        )}
      </div>

      {/* Verification Info */}
      <div style={{
        backgroundColor: theme.colors.surface,
        border: `1px solid ${theme.colors.primary}`,
        borderRadius: theme.borderRadius.md,
        padding: theme.spacing.lg,
        fontSize: '11px',
        lineHeight: '1.6',
      }}>
        <div style={{ marginBottom: theme.spacing.md, color: theme.colors.primary }}>
          VERIFICATION PROCESS:
        </div>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '10px' }}>
          <div>🔗 Hash Chain Verification</div>
          <div>⛓️ Blockchain Confirmation</div>
          <div>🔐 Cryptographic Integrity</div>
          <div>⏰ Timestamp Consistency</div>
          <div>📋 Chain of Custody</div>
          <div>⚖️ Court-Admissible Report</div>
          <div>🛡️ Tamper Evidence Detection</div>
          <div>📊 Quantified Assurance</div>
        </div>
      </div>
    </div>
  );
}