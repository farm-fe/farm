export const a = {
  field: '1',
};

export const b = window; // should be preserved

const Ani = window.Animation; // should be preserved
export const Animation = Ani; // should be preserved
Animation.prototype.run = function () {
  console.log('run');
}; // should be preserved