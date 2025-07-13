// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'provider.dart';

final getUserNameProvider = AutoDisposeFutureProvider<Future<String>>((ref) {
  return getUserName(ref);
});

final userNotifierProvider = NotifierProvider<UserNotifier, String>(() {
  return UserNotifier();
});
