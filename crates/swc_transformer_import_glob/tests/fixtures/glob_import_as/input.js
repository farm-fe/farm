const modules = import.meta.glob('./dir/*.js', { eager: true, as: 'raw' });

function loadImageUrls() {
  const images = import.meta.glob('./dir/*.js', { eager: true, as: 'url' });
  return images;
}