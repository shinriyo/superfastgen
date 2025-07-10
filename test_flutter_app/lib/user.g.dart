// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'user.dart';

abstract class _$User implements User{
  const _User();

  User copyWith({
    String? name,
    String? email,
    int? age,
  }) {
    return User(
      name: name ?? this.name,
      email: email ?? this.email,
      age: age ?? this.age,
    );
  }

  @override
  String toString() {
    return 'User(name: $name, email: $email, age: $age)';\n  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        other is _User &&
            name == other.name &&
            email == other.email &&
            age == other.age;
  }

  @override
  int get hashCode => name.hashCode ^ email.hashCode ^ age.hashCode;
}
