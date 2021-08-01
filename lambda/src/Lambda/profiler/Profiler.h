#ifndef LAMBDA_SRC_LAMBDA_PROFILER_PROFILER_H_
#define LAMBDA_SRC_LAMBDA_PROFILER_PROFILER_H_

#include <algorithm>
#include <fstream>
#include <thread>

#include <Lambda/lib/Time.h>

namespace lambda::profiler {

struct ProfileResult {
  std::string Name;
  lib::Time Start, Stop;
  std::thread::id ThreadID;

  ProfileResult(
      std::string name,
      lib::Time start,
      lib::Time stop,
      std::thread::id threadID) :
          Name(name),
          Start(start),
          Stop(stop),
          ThreadID(threadID) {}
};

struct ProfileSession {
  std::string Name;
};

/// @brief A built in micro profiler that allows us to measure execution times
/// of computation.
class Profiler {
 public:
  Profiler() : current_session_(nullptr), profile_count_(0) {}
  ~Profiler() {}

  /// @brief Write the header for the profile session.
  void WriteHeader() {
    output_stream_ << "{\"otherData\": {}, \"traceEvents\":[";
    output_stream_.flush();
  }

  /// @brief Write the footer of the profile session.
  void WriteFooter() {
    output_stream_ << "]}";
    output_stream_.flush();
  }

  /// @brief Begins the profile session.
  void BeginSession(
      const std::string& name,
      const std::string& file_path = "profile_results.json") {
    output_stream_.open(file_path);
    WriteHeader();
    current_session_ = new ProfileSession{ name };
  }

  /// @brief Ends the profile session.
  void EndSession() {
    WriteFooter();
    output_stream_.close();
    delete current_session_;
    current_session_ = nullptr;
    profile_count_ = 0;
  }

  /// @brief Write the result of a profile.
  void WriteProfile(const ProfileResult& result) {
    if (profile_count_++ > 0) {
      output_stream_ << ",";
    }

    std::string name = result.Name;
    std::replace(name.begin(), name.end(), '"', '\'');

    uint64_t start_time =
        result.Start.InMicroseconds().time_since_epoch().count();
    uint64_t end_time =
        result.Stop.InMicroseconds().time_since_epoch().count();

    output_stream_
        << "{"
        << "\"cat\":\"function\","
        << "\"dur\":" << (end_time - start_time) << ","
        << "\"name\":\"" << name << "\","
        << "\"ph\":\"X\","
        << "\"pid\":0,"
        << "\"tid\":" << result.ThreadID << ","
        << "\"ts\":" << start_time
        << "}";

    output_stream_.flush();
  }

  /// @brief Get a static instance of the profiler.
  static Profiler& Get() {
    static Profiler instance;
    return instance;
  }

 private:
  ProfileSession* current_session_;
  std::ofstream output_stream_;
  uint32_t profile_count_;
};

/// @brief A basic RAII timer used to profile computation within Lambda.
class Timer {
 public:
  explicit Timer(const char* name) : name_(name), stopped_(false), start_() {}

  ~Timer() {
    if (!stopped_) {
      Stop();
    }
  }

 private:
  bool stopped_;
  const char* name_;
  lib::Time start_;

  /// @brief Compute the duration of a function (Called within the Timers
  /// destructor.)
  void Stop() {
    lib::Time end;
    lib::TimeStep duration(start_, end);

    stopped_ = true;

    Profiler::Get().WriteProfile(
        { name_, start_, end, std::this_thread::get_id() });

    LAMBDA_CORE_INFO(
        "Duration of {0}: {1} ms", name_, duration.InMilliseconds<float>());
  }
};

#ifdef LAMBDA_INCLUDE_PROFILER
  /// @brief profile the current scope.
  #define LAMBDA_PROFILER_MEASURE_SCOPE(name) \
      ::lambda::profiler::Timer time##__LINE__(name)
  /// @brief Profile the current scope with the function name
  #define LAMBDA_PROFILER_MEASURE_FUNCTION() \
      LAMBDA_PROFILER_MEASURE_SCOPE(__FUNCTION__)
  /// @brief Begin a section that you'd like to profile.
  #define LAMBDA_PROFILER_BEGIN_SECTION(name) { \
      LAMBDA_PROFILER_MEASURE_SCOPE(name);
  /// @brief End the current section being profiled.
  #define LAMBDA_PROFILER_END_SECTION() }
  /// @brief Begin a profiling session.
  #define LAMBDA_PROFILER_BEGIN_SESSION(name, file_path) \
      ::lambda::profiler::Profiler::Get().BeginSession(name, file_path);
  /// @brief End a profiling session.
  #define LAMBDA_PROFILER_END_SESSION() \
      ::lambda::profiler::Profiler::Get().EndSession();
#else
  #define LAMBDA_PROFILER_MEASURE_SCOPE(name)
  #define LAMBDA_PROFILER_MEASURE_FUNCTION()
  #define LAMBDA_PROFILER_BEGIN_SECTION(name)
  #define LAMBDA_PROFILER_END_SECTION()
  #define LAMBDA_PROFILER_BEGIN_SESSION(name, file_path)
  #define LAMBDA_PROFILER_END_SESSION();
#endif

}  // namespace lambda::profiler

#endif  // LAMBDA_SRC_LAMBDA_PROFILER_PROFILER_H_
