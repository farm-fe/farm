<template>
  <t-space direction="vertical" style="width: 100%; text-align: center">
    <logo />
    <h3>Welcome to use TDesign！</h3>
    <t-space>
      <t-button theme="primary">
        <add-icon slot="icon" />
        新建
      </t-button>
      <t-button variant="outline">
        <cloud-upload-icon slot="icon" />
        上传文件
      </t-button>
      <t-button shape="circle" theme="primary">
        <discount-icon slot="icon" />
      </t-button>
      <t-button shape="circle" theme="primary">
        <cloud-download-icon slot="icon" />
      </t-button>
      <t-button variant="outline"> 搜索 </t-button>
    </t-space>
  </t-space>
</template>

<script>
import Vue from "vue";
import {
  AddIcon,
  CloudUploadIcon,
  DiscountIcon,
  CloudDownloadIcon,
} from "tdesign-icons-vue";
import Logo from "./assets/svg/assets-t-logo.svg?component";

export default Vue.extend({
  components: {
    AddIcon,
    CloudUploadIcon,
    DiscountIcon,
    CloudDownloadIcon,
    Logo,
  },
});
</script>