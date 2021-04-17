#ifndef LAMBDA_SRC_LAMBDA_LIB_FILE_H_
#define LAMBDA_SRC_LAMBDA_LIB_FILE_H_

#include <string>
#include <vector>
#include <mutex>

#include <Lambda/core/Core.h>
#include <Lambda/core/memory/Pointers.h>

namespace lambda::lib {

enum FileMode {
  None = BIT(0),
  Read = BIT(1),
  Write = BIT(2),
  Append = BIT(3),
  BinaryRead = BIT(4),
  BinaryWrite = BIT(5),
  BinaryAppend = BIT(6),
  BinaryCreateIfNotFound = BIT(7),
  CreateIfNotFound = BIT(8),
  OpenAtEnd = BIT(9),
  Truncate = BIT(10),
};

enum FilePosition {
  Current = 1,
  Beginning = 2,
  End = 3
};


/// TODO(C3NZ): Implement this for linux and (maybe) Windows system next.
// I will probably create a file store that will be used to sync files within a
// temp or local directory.
class File {
  // Create a static
 public:
  static core::memory::Shared<File> Create(
      const std::string path, FileMode mode);

  static core::memory::Shared<File> CreateTemp(
      const std::string& path, FileMode mode);

  static core::memory::Shared<File> Open(
      const std::string& path, FileMode mode);

  static void Delete(const std::string& path);

  virtual ~File();

  virtual void Close() = 0;

  // Check the modes
  virtual const bool CanRead() const = 0;
  virtual const bool CanWrite() const = 0;
  virtual const bool CanAppend() const = 0;
  virtual const bool IsBinary() const = 0;

  // Check the file status
  virtual const bool IsClosed() const = 0;
  virtual const bool IsTemp() const = 0;
  virtual const bool Exists() const  = 0;
  virtual const uintmax_t GetSize() const = 0;

  // Seek operations
  virtual uintmax_t GetCurrentPosition() = 0;
  virtual void SeekFromBeginning(uintmax_t bytes) = 0;
  virtual void SeekFromCurrent(uintmax_t bytes) = 0;
  virtual void SeekFromEnd(uintmax_t bytes) = 0;

  // Read and return strings
  virtual std::string Read(
      uint32_t bytes, FilePosition pos = FilePosition::Current) = 0;
  virtual void Read(
      char* buffer,
      uint32_t buffer_size,
      FilePosition pos = FilePosition::Current) = 0;

  virtual std::string ReadAll() = 0;
  virtual void ReadAll(char* buffer, uint32_t buffer_size) = 0;

  virtual std::string ReadLine(
      FilePosition position = FilePosition::Current) = 0;

  virtual std::vector<std::string> ReadLines(
      uint32_t lines, FilePosition position = FilePosition::Beginning) = 0;

  virtual void Write(
      const std::string& content,
      FilePosition position = FilePosition::Current) = 0;

  virtual void WriteLine(
      const std::string& line,
      FilePosition position = FilePosition::Current) = 0;

  virtual void WriteLines(
      const std::vector<std::string>& lines,
      FilePosition position = FilePosition::Current);

  // Save
  virtual void Save() = 0;
  virtual void SaveAndClose() = 0;

 protected:
  std::mutex lock;
};

}  // namespace lambda::lib

#endif  // LAMBDA_SRC_LAMBDA_LIB_FILE_H_
