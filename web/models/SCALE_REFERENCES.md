# Scale Reference Models

This directory contains scale reference models for the CAD viewer.

## Model Naming Convention

- `*.glb` - High-resolution models for visualization
- `ref_*.glb` - CAD-generated fallback models (from bvr/cad pipeline)

## Available References

### Banana (180mm)

| File             | Source         | Notes                          |
| ---------------- | -------------- | ------------------------------ |
| `banana.glb`     | High-res model | Primary visualization model    |
| `ref_banana.glb` | CAD pipeline   | Fallback (generated from Rust) |

### Astronaut (1850mm)

| File            | Source                             | Notes     |
| --------------- | ---------------------------------- | --------- |
| `astronaut.glb` | [Sketchfab](https://skfb.ly/oVwuz) | CC BY 4.0 |

**Attribution Required:**

> "astronaut" (https://skfb.ly/oVwuz) by Antropik is licensed under Creative Commons Attribution (http://creativecommons.org/licenses/by/4.0/).

### Grogu (34cm)

| File        | Source                                                                                          | Notes     |
| ----------- | ----------------------------------------------------------------------------------------------- | --------- |
| `grogu.glb` | [Sketchfab](https://sketchfab.com/3d-models/baby-yoda-free-3d-4bcdf59346944b3ebf94c714988a21db) | CC BY 4.0 |

**Attribution Required:**

> "BABY YODA FREE 3D BY OSCAR CREATIVO" (https://skfb.ly/6Rovs) by OSCAR CREATIVO is licensed under Creative Commons Attribution (http://creativecommons.org/licenses/by/4.0/).

## Adding New Scale References

1. Add the model file to this directory (`web/models/`)
2. Update `scaleReferences` array in `web/viewer.html`:

```javascript
{
  id: 'unique-id',
  name: 'Display Name',
  icon: '🎯',
  description: 'Size info',
  modelPath: 'models/yourmodel.glb',
  fallbackPath: null,  // or 'models/ref_yourmodel.glb'
  scale: 1.0,
  rotateY: Math.PI,
  material: null,  // Use model's materials, or specify custom
}
```

## Model Requirements

- Format: glTF 2.0 / GLB (binary)
- Units: Millimeters (consistent with CAD pipeline)
- Origin: Base of model at Y=0
- Orientation: Front facing +Z (will be rotated to face camera)

## Recommended Sources for Free Models

- [Sketchfab](https://sketchfab.com) (CC-licensed models)
- [Poly Haven](https://polyhaven.com)
- [Smithsonian 3D](https://3d.si.edu)
- [Thingiverse](https://thingiverse.com) (convert from STL)
