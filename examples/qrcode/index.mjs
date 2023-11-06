import QRCode from 'qrcode';

// With promises
QRCode.toCanvas('text', { errorCorrectionLevel: 'H' }, function (err, canvas) {
  if (err) throw err;

  var container = document.getElementById('canvas');
  container.appendChild(canvas);
});
