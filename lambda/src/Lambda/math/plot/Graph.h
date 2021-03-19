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
  Precision Min_x;
  Precision Max_x;
  Precision Min_z;
  Precision Max_y;
};

template<
  class Precision = lambda::math::Real,
  class Point = lambda::math::shapes::Point2D<Precision>,
  class Points = std::vector<Point>>
class Graph2D {
 public:
  /// @brief Construct a graph from a set of points.
  /// @param points The points to construct the graph with.
  explicit Graph2D(Points points);

  /// @brief Construct a graph from a configuration struct and the set of
  /// points to apply the config to.
  /// @param graph_config The configuration to use for the graph.
  /// @param points The points to utilize for constructing the grahp.
  Graph2D(GraphConfig<> graph_config, Points points);

  /// @brief The x position that the graph should start from.
  /// @param start_x Starting x position of the graph (Defaults to minimum x
  /// in the graph)
  /// @return The newly configured graph.
  Graph2D StartFrom(Precision start_x);

  /// @brief The x position that the graph should end at.
  /// @param end_x Ending x position of the graph (Defaults to the maximum x
  /// value in the graph.)
  /// @return The newly configured graph.
  Graph2D EndAt(Precision end_x);

  /// @brief The y position that the graph will display up to.
  /// @param upper_y Upper y position of the graph (Defaults to the maximum y
  /// value in the graph.)
  /// @return The newly configured graph.
  Graph2D UpTo(Precision upper_y);

  /// @brief The smallest y position that the graph will display down to.
  /// @param lower_y  Lower y position of the graph (Defaults to the minimum
  /// y value in the graph.)
  /// @return The newly configured graph.
  Graph2D DownTo(Precision lower_y);


 private:
  Points points_;
  int min_x_, max_x_, min_y_, max_y_;
};


}  // namespace lambda::math::plot

#endif  // LAMBDA_SRC_LAMBDA_MATH_PLOT_GRAPH_H_
