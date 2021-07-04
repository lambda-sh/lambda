#ifndef LAMBDA_SRC_LAMBDA_MATH_FACE_H_
#define LAMBDA_SRC_LAMBDA_MATH_FACE_H_

#include <Lambda/math/Vector3.h>

namespace lambda::math {

/// @brief The vector perpendicular to three vectors given the top,
/// right, and left side of the vector.
class Face {
 public:
  /// @brief Get the top, left, and right side of the vector.
  Face(Vector3 top, Vector3 left, Vector3 right)
    : top_(top), left_(left), right_(right), face_(
        CrossProductOf(left_ - top_, right_ - top_)) {}

  /// @brief Get the face computed from the top, left, and right vectors.
  const Vector3& GetFace() const {
    return face_;
  }

  /// @brief Get the top of the face.
  const Vector3& GetTop() const {
    return top_;
  }

  // @brief Get the bottom left vector of the face.
  const Vector3& GetLeft() const {
    return left_;
  }

  /// @brief Get the right vector of the face.
  const Vector3& GetRight() const {
    return right_;
  }

 private:
  Vector3 top_;
  Vector3 left_;
  Vector3 right_;
  Vector3 face_;
};

/// @brief Converts a face of 3D vectors into a 2D vectors.
///
/// Returns the face vectors in the order that a face is constructed. (Top,
/// left right.)
inline std::array<Vector2, 3> ToVector2(const Face& face) {
  return {
    ToVector2(face.GetTop()),
    ToVector2(face.GetLeft()),
    ToVector2(face.GetRight()),
  };
}

}  // namespace lambda::math

#endif  // LAMBDA_SRC_LAMBDA_MATH_FACE_H_
