import type { Ora } from 'ora';
import ora from 'ora';

const gradientColors = [
  `#afa0ff`,
  `#9b88ff`,
  `#a564ff`,
  `#974cff`,
  `#832aff`,
  `#afa0ff`,
  `#9b88ff`,
  `#a564ff`,
  `#974cff`,
  `#832aff`
];

// export const rocketAscii = '■■▶'
export const rocketAscii = '▶';

const referenceGradient = [
  ...gradientColors,
  ...[...gradientColors].reverse(),
  ...gradientColors
];

// async-friendly setTimeout
const sleep = (time: number) =>
  new Promise((resolve) => {
    setTimeout(resolve, time);
  });

function getGradientAnimFrames() {
  const frames = [];
  for (let start = 0; start < gradientColors.length * 2; start++) {
    const end = start + gradientColors.length - 1;
    frames.push(
      referenceGradient
        .slice(start, end)
        .map((g) => {
          return applyBackgroundColor(' ', g);
        })
        .join('')
    );
  }
  return frames;
}

function getIntroAnimFrames() {
  const frames = [];
  for (let end = 1; end <= gradientColors.length; end++) {
    const leadingSpacesArr = Array.from(
      new Array(Math.abs(gradientColors.length - end - 1)),
      () => ' '
    );
    const gradientArr = gradientColors
      .slice(0, end)
      .map((g) => applyBackgroundColor(' ', g));
    frames.push([...leadingSpacesArr, ...gradientArr].join(''));
  }
  return frames;
}

/**
 * Generate loading spinner with rocket flames!
 * @param text display text next to rocket
 * @returns Ora spinner for running .stop()
 */
export async function loadWithRocketGradient(text: string): Promise<Ora> {
  const frames = getIntroAnimFrames();
  const intro = ora({
    spinner: {
      interval: 30,
      frames
    },
    text: `${rocketAscii} ${text}`
  });
  intro.start();
  await sleep((frames.length - 1) * intro.interval);
  intro.stop();
  const spinner = ora({
    spinner: {
      interval: 80,
      frames: getGradientAnimFrames()
    },
    text: `${rocketAscii} ${text}`
  }).start();

  return spinner;
}

function applyBackgroundColor(text: string, hexColor: string) {
  // Ensure the hexColor is a valid hexadecimal color code
  const hexRegex = /^#([A-Fa-f0-9]{6}|[A-Fa-f0-9]{3})$/;
  if (!hexRegex.test(hexColor)) {
    throw new Error('Invalid hexadecimal color code');
  }

  // Add ANSI escape codes for background color
  const bgCode = `\x1b[48;2;${hexToRgb(hexColor).join(';')}m`;

  // Add ANSI reset code
  const resetCode = '\x1b[0m';

  // Return the formatted text
  return `${bgCode}${text}${resetCode}`;
}

// Helper function to convert hex color code to RGB
function hexToRgb(hex: string) {
  // Remove the hash character from the beginning, if present
  hex = hex.replace(/^#/, '');

  // Parse the hex string to obtain RGB values
  const bigint = parseInt(hex, 16);
  const r = (bigint >> 16) & 255;
  const g = (bigint >> 8) & 255;
  const b = bigint & 255;

  return [r, g, b];
}
