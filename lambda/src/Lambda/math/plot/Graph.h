/// @file Graph.h
/// @brief Implementation for graphs you can use for plotting.
#ifndef LAMBDA_SRC_LAMBDA_MATH_PLOT_GRAPH_H_
#define LAMBDA_SRC_LAMBDA_MATH_PLOT_GRAPH_H_

#include <string>
#include <vector>

#include <Lambda/concepts/Number.h>
#include <Lambda/concepts/Point.h>
#include <Lambda/math/shapes/Point.h>

namespace lambda::math::plot {

/// @brief Graph config for specifying the
template<concepts::NumberType Precision = lambda::math::Real>
struct Graph2DConfig {
  std::string Name;
  Precision From_x;
  Precision To_x;
  Precision Lower_y;
  Precision Upper_y;
};

/// @brief The graph parameters.
/// @tparam Precision The precision to use for the point system.
/// @tparam Point The point system to use for the graph.
/// @tparam Points The list of points to store the graph in (Must utilize
/// the same type as Point in the storage system.)
template<
  concepts::NumberType Precision = Real,
  concepts::PointType Point = shapes::Point2D<Precision>,
  concepts::PointContainer Points = std::vector<Point>>
class Graph2D {
 public:
  /// @brief Instantiate a graph from the pointer of another graph.
  /// @param graph
  explicit Graph2D(Graph2D* graph)
    : points_(graph->points_),
      from_x_(graph->from_x_),
      to_x_(graph->to_x_),
      upper_y_(graph->upper_y_),
      lower_y_(graph->lower_y_) {}

  /// @brief Construct a graph from a set of points.
  /// @param points The points to construct the graph with.
  explicit Graph2D(Points points) : points_(std::move(points)) {}

  /// @brief Construct a graph from a configuration struct and the set of
  /// points to apply the config to.
  /// @param points The points to utilize for constructing the graph.
  /// @param graph_config The configuration to use for the graph.
  Graph2D(Points points, Graph2DConfig<Precision> graph_config)
    : points_(points),
      from_x_(graph_config->From_x),
      to_x_(graph_config->To_x),
      upper_y_(graph_config->Upper_y),
      lower_y_(graph_config->Lower_y) {}


  /// @brief The x position that the graph should start from.
  /// @param from_x Starting x position of the graph (Defaults to minimum x
  /// in the graph)
  /// @return The newly configured graph.
  Graph2D StartFrom(Precision from_x) {
    from_x_ = from_x;
    return Graph2D(this);
  }

  /// @brief The x position that the graph should end at.
  /// @param to_x Ending x position of the graph (Defaults to the maximum x
  /// value in the graph.)
  /// @return The newly configured graph.
  Graph2D EndAt(Precision to_x) {
    to_x_ = to_x;
    return Graph2D(this);
  }

  /// @brief The y position that the graph will display up to.
  /// @param upper_y Upper y position of the graph (Defaults to the maximum y
  /// value in the graph.)
  /// @return The newly configured graph.
  Graph2D UpTo(Precision upper_y) {
    upper_y_ = upper_y;
    return Graph2D(this);
  }

  /// @brief The smallest y position that the graph will display down to.
  /// @param lower_y Lower y position of the graph (Defaults to the minimum
  /// y value in the graph.)
  /// @return The newly configured graph.
  Graph2D DownTo(Precision lower_y) {
    lower_y_ = lower_y;
    return Graph2D(this);
  }

  /// @brief Get a reference to the underlying point structure.
  [[nodiscard]] const Points& GetPoints() const {
    return points_;
  }

 private:
  Points points_;
  Precision from_x_, to_x_, upper_y_, lower_y_;
};


}  // namespace lambda::math::plot

#endif  // LAMBDA_SRC_LAMBDA_MATH_PLOT_GRAPH_H_
