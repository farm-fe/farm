import React, { Suspense, useRef } from "react";
import { Canvas, primitive, useLoader } from "@react-three/fiber";
import * as THREE from "three";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader";
import {
  Loader,
  useGLTF,
  OrbitControls,
  PerspectiveCamera,
  Environment,
  Stars,
} from "@react-three/drei";

import "./index.css";

function Model({ url }) {
  const { nodes, materials } = useGLTF(url);
  const model = useLoader(GLTFLoader, "/test.glb");

  return <primitive object={model.scene} />;
}

export default function App() {
  const starMaterial = new THREE.PointsMaterial({
    color: "rgb(255, 255, 0)", // 星星的颜色
    size: 0.1, // 星星的大小
  });

  const particleCount = 100;
  const particleGeometry = new THREE.BufferGeometry();
  const particleMaterial = new THREE.PointsMaterial({
    color: "white",
    size: 0.1,
  });
  const particlePositions = new Float32Array(particleCount * 3);

  for (let i = 0; i < particleCount; i++) {
    const x = Math.random() * 100 - 50;
    const y = Math.random() * 100 - 50;
    const z = Math.random() * 100 - 50;
    particlePositions[i * 3] = x;
    particlePositions[i * 3 + 1] = y;
    particlePositions[i * 3 + 2] = z;
  }

  particleGeometry.setAttribute(
    "position",
    new THREE.BufferAttribute(particlePositions, 3)
  );
  const particleSystem = useRef();
  return (
    <>
      <Canvas dpr={[1.5, 2]} linear shadows style={{ position: "absolute" }}>
        <fog attach="fog" args={["#272730", 16, 30]} />
        <ambientLight intensity={0.75} />
        <PerspectiveCamera makeDefault position={[0, 0, 16]} fov={75}>
          <pointLight intensity={1} position={[-10, -25, -10]} />
          <spotLight
            castShadow
            intensity={2.25}
            angle={0.2}
            penumbra={1}
            position={[-25, 20, -15]}
            shadow-mapSize={[1024, 1024]}
            shadow-bias={-0.0001}
          />
        </PerspectiveCamera>
        {/* <Suspense fallback={null}>
          <Model url="/test.glb" />
        </Suspense> */}
        <OrbitControls
          autoRotate
          autoRotateSpeed={0.015}
          enablePan={true}
          enableZoom={false}
          enableRotate={false}
          maxPolarAngle={Math.PI / 2}
          minPolarAngle={Math.PI / 2}
        />
        {/* <Stars radius={600} depth={50} count={1000} factor={150} /> */}
        <Stars
          radius={100} // 星空的半径
          // depth={50} // 星空的深度
          count={1500} // 星星的数量
          factor={8} // 星星的大小因子
          saturation={1} // 星星的饱和度
          fade // 星星是否渐隐
          // material={starMaterial} // 星星的材质
        />
      </Canvas>
      <div className="layer" />
      <Loader />
    </>
  );
}
