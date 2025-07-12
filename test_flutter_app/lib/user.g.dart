// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'user.dart';

User _$UserFromJson(Map<String, dynamic> json) {
  return _User(name: json['name'] as String, email: json['email'] as String, age: json['age'] as int?);
}

Map<String, dynamic> _$UserToJson(User instance) {
  return {
    'name': (instance as dynamic).name,
    'email': (instance as dynamic).email,
    'age': (instance as dynamic).age
  };
}
