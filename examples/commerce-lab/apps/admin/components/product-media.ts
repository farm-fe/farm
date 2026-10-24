import home from '../assets/product-home.svg?media';
import outdoor from '../assets/product-outdoor.svg?media';
import studio from '../assets/product-studio.svg?media';
import travel from '../assets/product-travel.svg?media';
const images: Record<string, string> = { Home: home, Outdoor: outdoor, Studio: studio, Travel: travel };
export function productImage(category: string) { return images[category] ?? home; }
