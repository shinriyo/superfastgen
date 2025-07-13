import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:riverpod/riverpod.dart';

part 'provider.g.dart';

@riverpod
Future<String> getUserName(Ref ref) async {
  // Simulate API call
  await Future.delayed(Duration(seconds: 1));
  return 'John Doe';
}

@riverpod
class UserNotifier extends Notifier<String> {
  @override
  String build() {
    return 'Initial state';
  }

  void updateName(String name) {
    state = name;
  }
}
