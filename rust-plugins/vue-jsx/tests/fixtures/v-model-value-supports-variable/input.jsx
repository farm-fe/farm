const foo = 'foo';

const a = () => 'a';

const b = { c: 'c' };
<>
  <A v-model={[xx, foo]} />
  <B v-model={[xx, ['a']]} />
  <C v-model={[xx, foo, ['a']]} />
  <D v-model={[xx, foo === 'foo' ? 'a' : 'b', ['a']]} />
  <E v-model={[xx, a(), ['a']]} />
  <F v-model={[xx, b.c, ['a']]} />
</>
