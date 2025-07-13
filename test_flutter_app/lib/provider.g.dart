// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'provider.dart';

final getUserNameProvider = AutoDisposeFutureProvider<String>((ref) {
  return getUserName(ref);
});

final getUserByIdProvider = AutoDisposeFutureProvider.family<String, String>((ref, params) {
  return getUserById(ref, params);
});

final getUsersByPageProvider = AutoDisposeFutureProvider.family<List<String>, Map<String, dynamic>>((ref, params) {
  return getUsersByPage(ref, page: params['page'], limit: params['limit']);
});

final userNotifierProvider = NotifierProvider<UserNotifier, String>(() {
  return UserNotifier();
});
