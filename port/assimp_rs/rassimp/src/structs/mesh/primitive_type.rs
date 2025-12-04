bitflags::bitflags! {
    /// Enumerates the types of geometric primitives supported by Assimp.
    /// @see aiFace Face data structure
    /// @see aiProcess_SortByPType Per-primitive sorting of meshes
    /// @see aiProcess_Triangulate Automatic triangulation
    /// @see AI_CONFIG_PP_SBP_REMOVE Removal of specific primitive types.
    ///
    #[derive(Clone, Copy, Debug, Default)]
    pub struct AiPrimitiveType: u8 {
        /**
         * @brief A point primitive.
         *
         * This is just a single vertex in the virtual world,
         * #aiFace contains just one index for such a primitive.
         */
        const Point = 0x1;

        /**
         * @brief A line primitive.
         *
         * This is a line defined through a start and an end position.
         * #aiFace contains exactly two indices for such a primitive.
         */
        const Line = 0x2;

        /**
         * @brief A triangular primitive.
         *
         * A triangle consists of three indices.
         */
        const Triangle = 0x4;

        /**
         * @brief A higher-level polygon with more than 3 edges.
         *
         * A triangle is a polygon, but polygon in this context means
         * "all polygons that are not triangles". The "Triangulate"-Step
         * is provided for your convenience, it splits all polygons in
         * triangles (which are much easier to handle).
         */
        const Polygon = 0x8;

        /**
         * @brief A flag to determine whether this triangles only mesh is NGON encoded.
         *
         * NGON encoding is a special encoding that tells whether 2 or more consecutive triangles
         * should be considered as a triangle fan. This is identified by looking at the first vertex index.
         * 2 consecutive triangles with the same 1st vertex index are part of the same
         * NGON.
         *
         * At the moment, only quads (concave or convex) are supported, meaning that polygons are 'seen' as
         * triangles, as usual after a triangulation pass.
         *
         * To get an NGON encoded mesh, please use the aiProcess_Triangulate post process.
         *
         * @see aiProcess_Triangulate
         * @link https://github.com/KhronosGroup/glTF/pull/1620
         */
        const NgonEncodingFlag = 0x10;
    }
}
