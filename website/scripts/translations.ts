import crowdin, {
  Credentials,
  TranslationStatus
} from '@crowdin/crowdin-api-client';
import fs from 'fs/promises';
import path from 'path';

const token =
  'eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9.eyJhdWQiOiJ3UUVxdmhVM3ZMT2EyWGljbVV5VCIsImp0aSI6IjYxZjI5YjkyYmU4YTUwMjk5MWEzMmM0ZWRmMDQ3ZTk1NjFhZDIyMzk2OGE1MmQ4MGFlNzM1YmY1YWVjNzZiMDVlODk5YjEzMWMzN2MxYWY2IiwiaWF0IjoxNjkwMjg5MTE2LjY2NDAxOCwibmJmIjoxNjkwMjg5MTE2LjY2NDAyMSwiZXhwIjoxNzIxODI1MTE2LjU1NjU4Miwic3ViIjoiMTU5MzQ5MDMiLCJzY29wZXMiOlsicHJvamVjdCJdLCJkb21haW4iOm51bGwsImFzc29jaWF0aW9ucyI6WyIqIl0sInNlc3Npb24iOjB9.rG2dtUOUI6y4YzlL4_AxfHSGhH8XsAQDBIPxb-hts56kOWnaACVHU9-Y2e7ABzaAmEsdxJ4Wl6kByFAF1DJ-n-ga02PVPW1EV8RulXZwram5lofZUjanq8OBuBMnOptJsxhmf962_t8G7lDvN0zYcOuddKg_sBcmEC1QS8ngF0OwDrQhOyzyAavciUIbwZERsLKoXyMj1EDWEtJ8UWePFcCiQUk5pffnOXR-lVgudO9JgK4breoKB6cTKp_J4mj8eFUmEOwgYhciNIxTYhmMrv43l9-sgPQA2NBct5p0ifHrPtktl5uEZYrjMQqwGhVDSVA5OakyYVdp2MsT2dTgBBqd9HY7Cuph9A5JDEL3ZwEW8Exw3qWcBDUKv7-IONcRSrbUIyTzw0ZYBSz5lE--33vsdWEOuZCDaU16pHOjCfudsiDjQI5RFVLYsMo05Qkf0zzse3WYNybxx2YrO2JiYRdjgGzN-inUoMek7phvjbFKq30XCI2CZDa3XrZjwgCF3RTAd-hoFIsi65u1vj-L554xvIU6NITHVklunZcIcZjoRFeDN1uw5pXcRMRcfchjBpSXNQRucdkRN-P2ay5dZS1M8tGlvygJtjc5pFgDGV51Zx_NFVGv75xrQ_UivKE8UIljwt5Aci_eHDE_f8keMANrATdSzMmXlyRUhh-8aWI';

(async () => {
  const credentials: Credentials = {
    token
  };
  const { projectsGroupsApi } = new crowdin(credentials);
  const t = await projectsGroupsApi.listProjects();
  const api: TranslationStatus = new TranslationStatus(credentials);
  const progress = await api.getProjectProgress(603701, { limit: 100 });
  const final = progress.data.reduce((acc, item) => {
    const { languageId, translationProgress, approvalProgress } = item.data;
    // @ts-ignore
    acc[languageId] = { translationProgress, approvalProgress };
    return acc;
  }, {});

  fs.writeFile(
    path.join(__dirname, './progress_translate_lang.json'),
    JSON.stringify(final, null, 4)
  );
})();
