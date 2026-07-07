import React from 'react';

type CrashVector = {
  label: string;
  counterExample: string;
  severity: 'High' | 'Critical';
};

const crashVectors: CrashVector[] = [
  { label: 'Division denominator proof', counterExample: 'z = 5 ⇒ (z - 5) = 0', severity: 'Critical' },
  { label: 'SQL construction policy', counterExample: 'sql_parameterized = false', severity: 'High' },
];

export function Dashboard() {
  return (
    <main style={{ background: '#FFFFFF', color: '#1F2937', minHeight: '100vh', padding: 48, fontFamily: 'Inter, -apple-system, BlinkMacSystemFont, sans-serif' }}>
      <section style={{ background: '#F9FAFB', border: '1px solid #E5E7EB', borderRadius: 28, padding: 32 }}>
        <p style={{ color: '#FF6B35', fontWeight: 700, letterSpacing: '0.08em', textTransform: 'uppercase' }}>Zevq AI Certification Console</p>
        <h1 style={{ fontSize: 48, lineHeight: 1.05, margin: '12px 0' }}>Mathematical transparency for AI safety audits.</h1>
        <p style={{ color: '#4B5563', maxWidth: 760 }}>Inspect parser findings, Z3 counter-examples, and deterministic safety states before generated code or agent tools reach production.</p>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, minmax(0, 1fr))', gap: 16, marginTop: 32 }}>
          {['Rust AST checks', 'Z3 bounded model checks', 'Air-gapped FFI bridge'].map((metric) => (
            <article key={metric} style={{ background: '#FFFFFF', border: '1px solid #E5E7EB', borderRadius: 20, padding: 20 }}>
              <strong>{metric}</strong>
              <p style={{ color: '#4B5563' }}>Active</p>
            </article>
          ))}
        </div>
      </section>
      <section style={{ display: 'grid', gap: 16, marginTop: 24 }}>
        {crashVectors.map((vector) => (
          <article key={vector.label} style={{ background: '#FF94B4', borderRadius: 22, color: '#1F2937', padding: 24 }}>
            <strong>{vector.severity}: {vector.label}</strong>
            <pre style={{ color: '#4B5563', whiteSpace: 'pre-wrap' }}>{vector.counterExample}</pre>
          </article>
        ))}
      </section>
    </main>
  );
}

export default Dashboard;
