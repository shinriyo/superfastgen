// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'provider.dart';

final getUserNameProvider = AutoDisposeFutureProvider<String>((ref) async {
  // Simulate API call
  await Future.delayed(Duration(seconds: 1));
  return 'John Doe';
});

abstract class _$UserNotifier extends AutoDisposeNotifier<String> {
  late final String _state;

  String get state => _state;

  @override
  String build() {
    _state = 'Initial state';
    return _state;
  }

  void updateName(String name) {
    _state = name;
    state = _state;
  }
}

final userNotifierProvider = AutoDisposeNotifierProvider<UserNotifier, String>(() {
  return UserNotifier();
});
