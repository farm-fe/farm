const modules = import.meta.glob('./dir/*.js', { import: 'setup' })

const modulesEager = import.meta.glob('./dir/*.js', { import: 'setup', eager: true })

const modulesDefault = import.meta.glob('./dir/*.js', { import: 'default' })


const modulesDefaultEager = import.meta.glob('./dir/*.js', { import: 'default', eager: true })