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
Future<String> getUserById(Ref ref, String userId) async {
  // Simulate API call with parameter
  await Future.delayed(Duration(seconds: 1));
  return 'User $userId';
}

@riverpod
Future<List<String>> getUsersByPage(Ref ref,
    {required int page, int limit = 10}) async {
  // Simulate API call with named parameters
  await Future.delayed(Duration(seconds: 1));
  return List.generate(limit, (i) => 'User ${page * limit + i}');
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
