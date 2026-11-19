export interface CalcResult {
  formatted: string;
  rawCopy: string;
  subtitle: string;
}

const CURRENCY_RATES_PER_USD: Record<string, { rate: number; symbol: string }> = {
  USD: { rate: 1, symbol: '$' },
  IDR: { rate: 15850, symbol: 'Rp ' },
  EUR: { rate: 0.92, symbol: '€' },
  GBP: { rate: 0.79, symbol: '£' },
  JPY: { rate: 154.2, symbol: '¥' },
  SGD: { rate: 1.34, symbol: 'S$' },
  AUD: { rate: 1.53, symbol: 'A$' },
  CAD: { rate: 1.39, symbol: 'C$' },
  KRW: { rate: 1395, symbol: '₩' },
  CNY: { rate: 7.24, symbol: '¥' },
};

const MASS_TO_KG: Record<string, number> = {
  kg: 1,
  g: 0.001,
  lb: 0.45359237,
  lbs: 0.45359237,
  oz: 0.02834952,
};

const SPEED_TO_MS: Record<string, number> = {
  'm/s': 1,
  ms: 1,
  kmh: 1 / 3.6,
  'km/h': 1 / 3.6,
  kph: 1 / 3.6,
  mph: 0.44704,
  knots: 0.514444,
};

const DIST_TO_M: Record<string, number> = {
  m: 1,
  km: 1000,
  cm: 0.01,
  mm: 0.001,
  mi: 1609.344,
  mile: 1609.344,
  miles: 1609.344,
  ft: 0.3048,
  in: 0.0254,
};

function formatNumber(n: number, decimals = 2): string {
  if (!Number.isFinite(n)) return 'Error';
  const abs = Math.abs(n);
  if (abs >= 1000 && Number.isInteger(Math.round(n * 100) / 100)) {
    return n.toLocaleString('en-US', { maximumFractionDigits: decimals });
  }
  return n.toLocaleString('en-US', {
    minimumFractionDigits: 0,
    maximumFractionDigits: decimals,
  });
}

/**
 * Safe recursive-descent math parser supporting +, -, *, /, ^, (), and %
 * Handles `(12500 * 14) - 15%` as `175000 * (1 - 0.15) = 148750`.
 */
function evaluateMath(expr: string): number | null {
  const cleaned = expr.replace(/,/g, '').replace(/\s+/g, '');
  if (!cleaned || !/^[0-9+\-*/^().%]+$/.test(cleaned)) return null;

  let pos = 0;

  const peek = () => cleaned[pos] ?? '';
  const consume = () => cleaned[pos++] ?? '';

  function parseExpression(): number {
    let left = parseTerm();
    while (peek() === '+' || peek() === '-') {
      const op = consume();
      const { value: right, isPercent } = parseTermWithPercent();
      if (isPercent) {
        left = op === '+' ? left * (1 + right / 100) : left * (1 - right / 100);
      } else {
        left = op === '+' ? left + right : left - right;
      }
    }
    return left;
  }

  function parseTermWithPercent(): { value: number; isPercent: boolean } {
    let left = parsePower();
    let isPercent = false;
    if (peek() === '%') {
      consume();
      isPercent = true;
    }
    while (peek() === '*' || peek() === '/') {
      isPercent = false;
      const op = consume();
      const right = parsePower();
      if (op === '*') left *= right;
      else {
        if (right === 0) throw new Error('Division by zero');
        left /= right;
      }
    }
    return { value: left, isPercent };
  }

  function parseTerm(): number {
    let left = parsePower();
    if (peek() === '%') {
      consume();
      left = left / 100;
    }
    while (peek() === '*' || peek() === '/') {
      const op = consume();
      let right = parsePower();
      if (peek() === '%') {
        consume();
        right = right / 100;
      }
      if (op === '*') left *= right;
      else {
        if (right === 0) throw new Error('Division by zero');
        left /= right;
      }
    }
    return left;
  }

  function parsePower(): number {
    const base = parseUnary();
    if (peek() === '^') {
      consume();
      const exp = parsePower();
      return Math.pow(base, exp);
    }
    return base;
  }

  function parseUnary(): number {
    if (peek() === '-') {
      consume();
      return -parseUnary();
    }
    if (peek() === '+') {
      consume();
      return parseUnary();
    }
    return parsePrimary();
  }

  function parsePrimary(): number {
    if (peek() === '(') {
      consume();
      const val = parseExpression();
      if (peek() !== ')') throw new Error('Unclosed parenthesis');
      consume();
      return val;
    }
    const start = pos;
    while ((peek() >= '0' && peek() <= '9') || peek() === '.') {
      consume();
    }
    if (pos === start) throw new Error('Unexpected token');
    const num = Number(cleaned.slice(start, pos));
    if (!Number.isFinite(num)) throw new Error('Invalid number');
    return num;
  }

  try {
    const result = parseExpression();
    if (pos !== cleaned.length || !Number.isFinite(result)) return null;
    return result;
  } catch {
    return null;
  }
}

export function evaluateSpotlightQuery(rawInput: string): CalcResult | null {
  const input = rawInput.trim();
  if (!input) return null;

  // 1. Currency or Unit conversion: "<amount> <from> to|in <to>"
  const convMatch = input.match(
    /^([+-]?\d[\d,]*(?:\.\d+)?)\s*([a-zA-Z/°]+)\s+(?:to|in|->)\s+([a-zA-Z/°]+)$/i
  );
  if (convMatch) {
    const amount = Number(convMatch[1].replace(/,/g, ''));
    const fromRaw = convMatch[2].toLowerCase().replace('°', '');
    const toRaw = convMatch[3].toLowerCase().replace('°', '');
    const fromUpper = fromRaw.toUpperCase();
    const toUpper = toRaw.toUpperCase();

    if (Number.isFinite(amount)) {
      // Currency
      if (CURRENCY_RATES_PER_USD[fromUpper] && CURRENCY_RATES_PER_USD[toUpper]) {
        const usd = amount / CURRENCY_RATES_PER_USD[fromUpper].rate;
        const converted = usd * CURRENCY_RATES_PER_USD[toUpper].rate;
        const decimals = toUpper === 'IDR' || toUpper === 'JPY' || toUpper === 'KRW' ? 0 : 2;
        const formattedNum = formatNumber(converted, decimals);
        const sym = CURRENCY_RATES_PER_USD[toUpper].symbol;
        return {
          formatted: `${sym}${formattedNum}`,
          rawCopy: `${formattedNum} ${toUpper}`,
          subtitle: `${formatNumber(amount)} ${fromUpper} → ${toUpper} (FX ref)`,
        };
      }

      // Mass
      if (MASS_TO_KG[fromRaw] && MASS_TO_KG[toRaw]) {
        const kg = amount * MASS_TO_KG[fromRaw];
        const out = kg / MASS_TO_KG[toRaw];
        const formattedNum = formatNumber(out, 2);
        return {
          formatted: `${formattedNum} ${toRaw}`,
          rawCopy: `${formattedNum} ${toRaw}`,
          subtitle: `Mass conversion · ${amount} ${fromRaw}`,
        };
      }

      // Speed
      if (SPEED_TO_MS[fromRaw] && SPEED_TO_MS[toRaw]) {
        const ms = amount * SPEED_TO_MS[fromRaw];
        const out = ms / SPEED_TO_MS[toRaw];
        const formattedNum = formatNumber(out, 2);
        return {
          formatted: `${formattedNum} ${toRaw}`,
          rawCopy: `${formattedNum} ${toRaw}`,
          subtitle: `Speed conversion · ${amount} ${fromRaw}`,
        };
      }

      // Distance
      if (DIST_TO_M[fromRaw] && DIST_TO_M[toRaw]) {
        const meters = amount * DIST_TO_M[fromRaw];
        const out = meters / DIST_TO_M[toRaw];
        const formattedNum = formatNumber(out, 2);
        return {
          formatted: `${formattedNum} ${toRaw}`,
          rawCopy: `${formattedNum} ${toRaw}`,
          subtitle: `Distance conversion · ${amount} ${fromRaw}`,
        };
      }

      // Temperature
      if (['c', 'f', 'k'].includes(fromRaw) && ['c', 'f', 'k'].includes(toRaw)) {
        let celsius = amount;
        if (fromRaw === 'f') celsius = ((amount - 32) * 5) / 9;
        if (fromRaw === 'k') celsius = amount - 273.15;
        let out = celsius;
        if (toRaw === 'f') out = (celsius * 9) / 5 + 32;
        if (toRaw === 'k') out = celsius + 273.15;
        const formattedNum = formatNumber(out, 1);
        return {
          formatted: `${formattedNum}°${toUpper}`,
          rawCopy: `${formattedNum}°${toUpper}`,
          subtitle: `Temperature · ${amount}°${fromUpper}`,
        };
      }
    }
  }

  // 2. Math expression
  const mathVal = evaluateMath(input);
  if (mathVal !== null) {
    const formattedNum = formatNumber(mathVal, 4);
    return {
      formatted: formattedNum,
      rawCopy: String(Math.round(mathVal * 10000) / 10000),
      subtitle: 'Math expression',
    };
  }

  return null;
}
