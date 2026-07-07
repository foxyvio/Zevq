import type { NextApiRequest, NextApiResponse } from 'next';

type AuditStatus = 'PASS' | 'CRASH';

type AuditResponse = {
  status: AuditStatus;
  engine: 'wizard-of-oz-z3-emulator';
  sanitizedPayload: string;
  counterExample?: {
    variable: string;
    value: number;
    proof: string;
  };
};

const DIVISION_MINUS_PATTERN = /\/\s*\([^)]*\w+\s*-\s*\d+[^)]*\)/;

function sanitizePayload(payload: unknown): string {
  return String(payload ?? '')
    .replace(/[\u0000-\u001F\u007F]/g, '')
    .slice(0, 20_000);
}

export default function handler(req: NextApiRequest, res: NextApiResponse<AuditResponse>) {
  if (req.method !== 'POST') {
    res.setHeader('Allow', 'POST');
    res.status(405).json({
      status: 'CRASH',
      engine: 'wizard-of-oz-z3-emulator',
      sanitizedPayload: '',
      counterExample: {
        variable: 'method',
        value: 405,
        proof: 'Only POST telemetry submissions are accepted.',
      },
    });
    return;
  }

  const sanitizedPayload = sanitizePayload(req.body?.payload ?? req.body);

  if (DIVISION_MINUS_PATTERN.test(sanitizedPayload)) {
    res.status(200).json({
      status: 'CRASH',
      engine: 'wizard-of-oz-z3-emulator',
      sanitizedPayload,
      counterExample: {
        variable: 'z',
        value: 5,
        proof: 'Emulated Z3 model: denominator (z - 5) becomes zero when z equals 5.',
      },
    });
    return;
  }

  res.status(200).json({
    status: 'PASS',
    engine: 'wizard-of-oz-z3-emulator',
    sanitizedPayload,
  });
}
