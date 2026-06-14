const Component = (row) => (
  <NSpace>
    <NButton
      type="primary"
      secondary
      onClick={handler1}
    >
      {t('text1')}
    </NButton>
    <NButton onClick={handler2}>
      {t('text2')}
    </NButton>
    <NButton type="error" onClick={handler3}>
      {t('text3')}
    </NButton>
  </NSpace>
)
