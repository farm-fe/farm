import "./style.css";
  
  // Individual imports for each component
  import "@arcgis/map-components/dist/components/arcgis-map";
  import "@arcgis/map-components/dist/components/arcgis-legend";
  import "@arcgis/map-components/dist/components/arcgis-search";

  const mapElement = document.querySelector('arcgis-map');

  let isReady: boolean[] = [];
  
  mapElement?.addEventListener('arcgisViewReadyChange', event => {
    console.log('MapView ready', event);
    isReady.push(event.target.updating);
  });
  mapElement?.addEventListener('arcgisViewLayerviewCreate', event => {
    console.log('arcgisViewLayerviewCreate updating', event, event.target.updating);
    isReady.push(event.target.updating);

    if (isReady.length === 5 && isReady.every(item => item)) {
      console.log('arcgis all ready');
    }
  });