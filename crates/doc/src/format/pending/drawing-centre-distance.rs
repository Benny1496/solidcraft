//! Experimental drawing annotations may store kind `center_distance`, two verified
//! face names (`face_name`, `second_face_name`) and their measured XY centres
//! inside `org.solidcraft.experimental_drawing` document metadata. Older drawings
//! have no centre-pair annotations and need no rewrite. A released reader refuses
//! the newer pending design format instead of interpreting unsupported dimensions.
