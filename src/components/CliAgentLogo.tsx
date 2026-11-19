import React from 'react';
import { Terminal } from 'lucide-react';

interface CliAgentLogoProps {
  agentName: string;
  className?: string;
  color?: string;
}

/**
 * Official Anthropic Spark / Starburst SVG
 * Clean minimalist geometric glyph in warm coral (#CC785C) or clean white.
 */
export const AnthropicSpark: React.FC<{ className?: string; color?: string }> = ({
  className = 'w-3.5 h-3.5',
  color = '#CC785C',
}) => (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    className={className}
    xmlns="http://www.w3.org/2000/svg"
  >
    {/* 8-spoke Anthropic Asterisk / Spark */}
    <path
      d="M12 2.5V8.5M12 15.5V21.5M2.5 12H8.5M15.5 12H21.5M5.28 5.28L9.52 9.52M14.48 14.48L18.72 18.72M5.28 18.72L9.52 14.48M14.48 9.52L18.72 5.28"
      stroke={color}
      strokeWidth="2.75"
      strokeLinecap="round"
    />
  </svg>
);

/**
 * Minimalist OpenAI / Codex Spiral Icon
 */
export const CodexSpiral: React.FC<{ className?: string; color?: string }> = ({
  className = 'w-3.5 h-3.5',
  color = '#10A37F',
}) => (
  <svg
    viewBox="0 0 24 24"
    fill="none"
    className={className}
    stroke={color}
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
    xmlns="http://www.w3.org/2000/svg"
  >
    <path d="M12 3a9 9 0 0 1 9 9c0 4.97-4.03 9-9 9s-9-4.03-9-9" />
    <path d="M12 7a5 5 0 0 1 5 5c0 2.76-2.24 5-5 5s-5-2.24-5-5" />
    <circle cx="12" cy="12" r="1.5" fill={color} />
  </svg>
);

/**
 * CLI Agent Logo Resolver
 * Renders official brand logos without generic robot emojis:
 * - Claude Code -> Anthropic Spark
 * - Codex -> OpenAI / Codex Spiral
 * - Generic -> Lucide Terminal
 */
export const CliAgentLogo: React.FC<CliAgentLogoProps> = ({
  agentName,
  className = 'w-3.5 h-3.5',
  color,
}) => {
  const lower = agentName.toLowerCase();
  if (lower.includes('claude')) {
    return <AnthropicSpark className={className} color={color ?? '#CC785C'} />;
  }
  if (lower.includes('codex') || lower.includes('openai')) {
    return <CodexSpiral className={className} color={color ?? '#10A37F'} />;
  }
  return <Terminal className={`${className} text-white/80`} />;
};
