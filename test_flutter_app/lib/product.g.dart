// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'product.dart';

Product _$ProductFromJson(Map<String, dynamic> json) {
  return Product(id: json['id'] as String, name: json['name'] as String, price: json['price'] as double, description: json['description'] as String?);
}

Map<String, dynamic> _$ProductToJson(Product instance) {
  return {
    'id': (instance as dynamic).id,
    'name': (instance as dynamic).name,
    'price': (instance as dynamic).price,
    'description': (instance as dynamic).description
  };
}
