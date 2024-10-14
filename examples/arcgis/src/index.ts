import "./style.css";
  
  // Individual imports for each component
  import "@arcgis/map-components/dist/components/arcgis-map";
  import "@arcgis/map-components/dist/components/arcgis-legend";
  import "@arcgis/map-components/dist/components/arcgis-search";

  const mapElement = document.querySelector('arcgis-map');
  mapElement.addEventListener('arcgisViewReadyChange', event => {
    console.log('MapView ready', event);
  });