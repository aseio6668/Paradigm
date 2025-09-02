// Three.js JSX elements for React Three Fiber
declare global {
  namespace JSX {
    interface IntrinsicElements {
      // Three.js Object3D elements
      group: any
      mesh: any
      points: any
      line: any
      
      // Three.js Material elements  
      meshPhongMaterial: any
      pointsMaterial: any
      lineBasicMaterial: any
      
      // Three.js Light elements
      ambientLight: any
      pointLight: any
      directionalLight: any
      spotLight: any
      
      // Three.js Geometry elements
      boxGeometry: any
      sphereGeometry: any
      planeGeometry: any
      bufferGeometry: any
    }
  }
}

export {};