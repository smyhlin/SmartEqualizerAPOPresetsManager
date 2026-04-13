const CONVOLUTION_LINE_RE = /^(\s*Convolution\s*:\s*)([^\r\n]*)/im;
const CONVOLUTION_LINE_WITH_ENDING_RE = /^(\s*Convolution\s*:\s*[^\r\n]*)(\r?\n)?/im;

function stripWrappingQuotes(value: string) {
  const trimmed = value.trim();

  if (
    (trimmed.startsWith('"') && trimmed.endsWith('"')) ||
    (trimmed.startsWith("'") && trimmed.endsWith("'"))
  ) {
    return trimmed.slice(1, -1).trim();
  }

  return trimmed;
}

function formatConvolutionPath(path: string) {
  return `"${path.trim().replaceAll('"', '\\"')}"`;
}

export function extractConvolutionPath(content: string) {
  const match = content.match(CONVOLUTION_LINE_RE);
  if (!match) {
    return null;
  }

  const rawPath = stripWrappingQuotes(match[2]);
  return rawPath.length > 0 ? rawPath : null;
}

export function replaceConvolutionPath(content: string, wavPath: string) {
  const line = `Convolution: ${formatConvolutionPath(wavPath)}`;

  if (CONVOLUTION_LINE_RE.test(content)) {
    return content.replace(CONVOLUTION_LINE_RE, `$1${formatConvolutionPath(wavPath)}`);
  }

  if (content.length === 0) {
    return `${line}\n`;
  }

  const separator = /\r?\n$/.test(content) ? '' : '\n';
  return `${content}${separator}${line}\n`;
}

export function removeConvolutionPath(content: string) {
  return content.replace(CONVOLUTION_LINE_WITH_ENDING_RE, '');
}

export function buildConvolutionPresetContent(wavPath: string) {
  return `${`Convolution: ${formatConvolutionPath(wavPath)}`}\r\n`;
}
