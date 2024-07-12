<template>
  <t-space direction="vertical" style="width: 100%; text-align: center">
    <logo />
    <img :src="farmLogo" alt="farm logo" style="width: 100px" />
    {{ res }}
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
import farmLogo from "./assets/logo.png";
import { res } from "@/index";
import axios from "axios";
export default {
  data() {
    return {
      res,
      farmLogo,
    };
  },
  components: {
    AddIcon,
    CloudUploadIcon,
    DiscountIcon,
    CloudDownloadIcon,
    Logo,
  },
  created() {
    axios.get("/aaa?key=caab07f6c5df752d2e28edfe447ae6d0").then((res) => {
      console.log(res);
    });
  },
};
</script>
