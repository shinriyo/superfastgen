// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'product.dart';

Product _$ProductFromJson(Map<String, dynamic> json) {
  return Product(
    id: json['id'] as String,
    name: json['name'] as String,
    price: (json['price'] as num).toDouble(),
    description: json['description'] as String?,
  );
}

Map<String, dynamic> _$ProductToJson(Product instance) {
  return <String, dynamic>{
    'id': instance.id,
    'name': instance.name,
    'price': instance.price,
    'description': instance.description,
  };
}
