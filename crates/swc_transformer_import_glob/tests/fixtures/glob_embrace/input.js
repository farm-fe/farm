const modules = import.meta.glob('./dir/*.{js,cjs,mjs}', { import: 'setup' })

const modulesEager = import.meta.glob('./dir/*.{js,cjs,mjs}', { import: 'setup', eager: true })

const modulesDefault = import.meta.glob('./dir/*.{js,cjs,mjs}', { import: 'default' })


const modulesDefaultEager = import.meta.glob('./dir/*.{js,cjs,mjs}', { import: 'default', eager: true })