import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';

typedef _AssessNative = Pointer<Utf8> Function(Pointer<Utf8> input);
typedef _AssessDart = Pointer<Utf8> Function(Pointer<Utf8> input);
typedef _FreeNative = Void Function(Pointer<Utf8> ptr);
typedef _FreeDart = void Function(Pointer<Utf8> ptr);

class ZevqRustBridge {
  ZevqRustBridge({DynamicLibrary? library}) : _library = library ?? _openLibrary();

  final DynamicLibrary _library;

  String assessStartup(String profileJsonOrName) {
    final assess = _library.lookupFunction<_AssessNative, _AssessDart>('zevq_assess_startup');
    final release = _library.lookupFunction<_FreeNative, _FreeDart>('zevq_free_string');
    final input = profileJsonOrName.toNativeUtf8();
    final output = assess(input);
    try {
      return output.toDartString();
    } finally {
      calloc.free(input);
      release(output);
    }
  }

  static DynamicLibrary _openLibrary() {
    if (Platform.isMacOS || Platform.isIOS) {
      return DynamicLibrary.open('libzevq_backend.dylib');
    }
    if (Platform.isWindows) {
      return DynamicLibrary.open('zevq_backend.dll');
    }
    return DynamicLibrary.open('libzevq_backend.so');
  }
}
