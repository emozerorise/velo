import { describe, expect, it } from 'vitest';
import { providerHost, providerIsRemote } from '@/utils/providerLocation';

describe('summary provider location', () => {
  it('recognises only actual loopback hosts as local', () => {
    expect(providerIsRemote('http://localhost:11434')).toBe(false);
    expect(providerIsRemote('http://127.0.0.1:11434/v1')).toBe(false);
    expect(providerIsRemote('http://[::1]:11434')).toBe(false);
    expect(providerIsRemote('https://localhost.example.com/v1')).toBe(true);
    expect(providerIsRemote('http://192.168.1.20:11434')).toBe(true);
  });

  it('returns the visible destination host including its port', () => {
    expect(providerHost('https://api.example.com:8443/v1')).toBe('api.example.com:8443');
    expect(providerHost('not a URL')).toBe('not a URL');
  });
});
