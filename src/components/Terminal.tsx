import { useEffect, useRef } from 'react';
import { theme } from '../theme';

interface TerminalProps {
  log: string[];
}

export function Terminal({ log }: TerminalProps) {
  const terminalRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (terminalRef.current) {
      terminalRef.current.scrollTop = terminalRef.current.scrollHeight;
    }
  }, [log]);

  return (
    <div style={{
      height: '100%',
      backgroundColor: theme.colors.background,
      border: `1px solid ${theme.colors.primary}`,
      borderRadius: theme.borderRadius.md,
      padding: theme.spacing.md,
      fontFamily: 'monospace',
      fontSize: '12px',
      overflow: 'auto',
    }}>
      <div
        ref={terminalRef}
        style={{
          whiteSpace: 'pre-wrap',
          lineHeight: '1.4',
          color: theme.colors.text,
          textShadow: `0 0 3px ${theme.colors.primary}`,
        }}
      >
        {log.map((line, index) => (
          <div key={index} style={{
            marginBottom: '2px',
            opacity: line.startsWith('>') ? 1 : 0.8,
          }}>
            {line}
          </div>
        ))}
        
        {/* Cursor */}
        <div style={{
          display: 'inline-block',
          width: '8px',
          height: '14px',
          backgroundColor: theme.colors.primary,
          animation: 'blink 1s infinite',
        }} />
      </div>
      
      <style>{`
        @keyframes blink {
          0%, 50% { opacity: 1; }
          51%, 100% { opacity: 0; }
        }
      `}</style>
    </div>
  );
}