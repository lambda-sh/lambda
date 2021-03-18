#ifndef LAMBDA_SRC_LAMBDA_MATH_PLOT_GRAPH_H_
#define LAMBDA_SRC_LAMBDA_MATH_PLOT_GRAPH_H_

#include <string>
#include <vector>

#include <Lambda/math/shapes/Point.h>

namespace lambda::math::plot {

/// @brief Graph config for specifying the
template<class Precision = lambda::math::Real>
struct GraphConfig {
  std::string Name;
  Precision Minimum_X_;
  Precision Maximum_X_;
  Precision Minimum_Y_;
  Precision Maximum_Y_;
};

template<class Points = lambda::math::shapes::Point2D<lambda::math::Real>>
class Graph {
 public:
  explicit Graph(std::vector<Points> points) : points_(points) {}
 private:
  std::vector<Points> points_;
  int min_x_, max_x_, min_y_, max_y_;
};


}  // namespace lambda::math::plot

#endif  // LAMBDA_SRC_LAMBDA_MATH_PLOT_GRAPH_H_
