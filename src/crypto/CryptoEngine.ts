// @ts-nocheck
// Core cryptographic primitives using Web Crypto API
export class CryptoEngine {
  private static instance: CryptoEngine;
  private algorithmKey = 'AES-GCM';
  private algorithmHash = 'SHA-256';

  private constructor() {}

  static getInstance(): CryptoEngine {
    if (!CryptoEngine.instance) {
      CryptoEngine.instance = new CryptoEngine();
    }
    return CryptoEngine.instance;
  }

  // Generate a cryptographically secure random key
  async generateKey(): Promise<CryptoKey> {
    return await crypto.subtle.generateKey(
      {
        name: this.algorithmKey,
        length: 256,
      },
      true,
      ['encrypt', 'decrypt']
    );
  }

  // Export key to base64
  async exportKey(key: CryptoKey): Promise<string> {
    const exported = await crypto.subtle.exportKey('raw', key);
    const bytes = new Uint8Array(exported);
    return btoa(String.fromCharCode(...bytes));
  }

  // Import key from base64
  async importKey(base64Key: string): Promise<CryptoKey> {
    const keyData = new Uint8Array(atob(base64Key).split('').map(c => c.charCodeAt(0)));
    return await crypto.subtle.importKey(
      'raw',
      keyData,
      this.algorithmKey,
      true,
      ['encrypt', 'decrypt']
    );
  }

  // Encrypt data with authenticated encryption
  async encrypt(data: string, key: CryptoKey): Promise<{ ciphertext: Uint8Array; iv: Uint8Array; tag: Uint8Array }> {
    const iv = crypto.getRandomValues(new Uint8Array(12));
    const encoder = new TextEncoder();
    const encodedData = encoder.encode(data);
    
    const encrypted = await crypto.subtle.encrypt(
      {
        name: this.algorithmKey,
        iv: iv,
      },
      key,
      encodedData
    );

    const encryptedArray = new Uint8Array(encrypted);
    const ciphertext = encryptedArray.slice(0, -16);
    const tag = encryptedArray.slice(-16);

    return { ciphertext, iv, tag };
  }

  // Decrypt data with authentication verification
  async decrypt(ciphertext: Uint8Array, iv: Uint8Array, tag: Uint8Array, key: CryptoKey): Promise<string> {
    const combinedData = new Uint8Array(ciphertext.length + tag.length);
    combinedData.set(ciphertext);
    combinedData.set(tag, ciphertext.length);

    const decrypted = await crypto.subtle.decrypt(
      {
        name: this.algorithmKey,
        iv: iv,
      },
      key,
      combinedData
    );

    const decoder = new TextDecoder();
    return decoder.decode(decrypted);
  }

  // Generate cryptographic hash
  async hash(data: string): Promise<string> {
    const encoder = new TextEncoder();
    const dataArray = encoder.encode(data);
    const hashBuffer = await crypto.subtle.digest(this.algorithmHash, dataArray);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
  }

  // Generate HMAC for message authentication
  async hmac(data: string, key: CryptoKey): Promise<string> {
    const encoder = new TextEncoder();
    const dataArray = encoder.encode(data);
    const hmacBuffer = await crypto.subtle.sign('HMAC', key, dataArray);
    const hmacArray = Array.from(new Uint8Array(hmacBuffer));
    return hmacArray.map(b => b.toString(16).padStart(2, '0')).join('');
  }

  // Generate HMAC key
  async generateHmacKey(): Promise<CryptoKey> {
    return await crypto.subtle.generateKey(
      {
        name: 'HMAC',
        hash: this.algorithmHash,
      },
      true,
      ['sign', 'verify']
    );
  }

  // Import HMAC key from base64
  async importHmacKey(base64Key: string): Promise<CryptoKey> {
    const keyData = new Uint8Array(atob(base64Key).split('').map(c => c.charCodeAt(0)));
    return await crypto.subtle.importKey(
      'raw',
      keyData,
      {
        name: 'HMAC',
        hash: this.algorithmHash,
      },
      true,
      ['sign', 'verify']
    );
  }
}