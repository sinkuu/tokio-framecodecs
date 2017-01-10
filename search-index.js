var searchIndex = {};
searchIndex["framecodecs"] = {"doc":"`framecodecs` provides simple protocol implementations to be used with `tokio_proto::pipeline`.","items":[[0,"fixed_length","framecodecs","Fixed-length protocol.",null,null],[3,"FixedLengthProto","framecodecs::fixed_length","A protocol such that frames are continuous and have the same specified length.",null,null],[12,"length","","",0,null],[3,"FixedLengthCodec","","Protocol codec used by `FixedLengthProto`.",null,null],[11,"fmt","","",0,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"clone","","",0,{"inputs":[{"name":"self"}],"output":{"name":"fixedlengthproto"}}],[11,"eq","","",0,{"inputs":[{"name":"self"},{"name":"fixedlengthproto"}],"output":{"name":"bool"}}],[11,"ne","","",0,{"inputs":[{"name":"self"},{"name":"fixedlengthproto"}],"output":{"name":"bool"}}],[11,"new","","",0,{"inputs":[{"name":"usize"}],"output":{"name":"fixedlengthproto"}}],[11,"bind_transport","","",0,null],[11,"bind_transport","","",0,null],[11,"fmt","","",1,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"clone","","",1,{"inputs":[{"name":"self"}],"output":{"name":"fixedlengthcodec"}}],[11,"new","","",1,{"inputs":[{"name":"usize"}],"output":{"name":"fixedlengthcodec"}}],[11,"length","","",1,{"inputs":[{"name":"self"}],"output":{"name":"usize"}}],[11,"decode","","",1,{"inputs":[{"name":"self"},{"name":"easybuf"}],"output":{"name":"result"}}],[11,"encode","","",1,{"inputs":[{"name":"self"},{"name":"vec"},{"name":"vec"}],"output":{"name":"result"}}],[0,"delimiter","framecodecs","Delimitered protocol.",null,null],[3,"DelimiterProto","framecodecs::delimiter","A protocol such that frames are separated with specified delimiters.",null,null],[3,"DelimiterCodec","","Protocol codec used by `DelimiterProto`.",null,null],[4,"LineDelimiter","","A line break delimiter.",null,null],[13,"Cr","","Carriage return.",2,null],[13,"Lf","","Line feed.",2,null],[13,"CrLf","","Carriage return / line feed.",2,null],[8,"Delimiter","","A delimiter.",null,null],[10,"pop_buf","","Removes elements from buffer including next occurence of this delimiter, and returns the removed part except the delimiter.",3,{"inputs":[{"name":"self"},{"name":"easybuf"}],"output":{"name":"result"}}],[10,"write_delimiter","","Appends this delimiter to the buffer.",3,{"inputs":[{"name":"self"},{"name":"vec"}],"output":null}],[11,"fmt","","",4,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"clone","","",4,{"inputs":[{"name":"self"}],"output":{"name":"delimiterproto"}}],[11,"eq","","",4,{"inputs":[{"name":"self"},{"name":"delimiterproto"}],"output":{"name":"bool"}}],[11,"ne","","",4,{"inputs":[{"name":"self"},{"name":"delimiterproto"}],"output":{"name":"bool"}}],[11,"new","","Creates a `DelimiterProto` from the specified delimiter.",4,{"inputs":[{"name":"d"}],"output":{"name":"self"}}],[11,"bind_transport","","",4,null],[11,"bind_transport","","",4,null],[11,"fmt","","",5,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"clone","","",5,{"inputs":[{"name":"self"}],"output":{"name":"delimitercodec"}}],[11,"new","","",5,{"inputs":[{"name":"d"}],"output":{"name":"delimitercodec"}}],[11,"decode","","",5,{"inputs":[{"name":"self"},{"name":"easybuf"}],"output":{"name":"result"}}],[11,"encode","","",5,{"inputs":[{"name":"self"},{"name":"vec"},{"name":"vec"}],"output":{"name":"result"}}],[11,"fmt","","",2,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"clone","","",2,{"inputs":[{"name":"self"}],"output":{"name":"linedelimiter"}}],[11,"eq","","",2,{"inputs":[{"name":"self"},{"name":"linedelimiter"}],"output":{"name":"bool"}}],[11,"pop_buf","","",2,{"inputs":[{"name":"self"},{"name":"easybuf"}],"output":{"name":"result"}}],[11,"write_delimiter","","",2,{"inputs":[{"name":"self"},{"name":"vec"}],"output":null}],[0,"length_field","framecodecs","Length field prepending protocol.",null,null],[3,"LengthFieldProto","framecodecs::length_field","A protocol such that every frame has length field prepended in specified size and byte-order.",null,null],[12,"field_size","","",6,null],[3,"LengthFieldCodec","","Protocol codec used by `LengthFieldProto`.",null,null],[11,"fmt","","",6,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"clone","","",6,{"inputs":[{"name":"self"}],"output":{"name":"lengthfieldproto"}}],[11,"eq","","",6,{"inputs":[{"name":"self"},{"name":"lengthfieldproto"}],"output":{"name":"bool"}}],[11,"ne","","",6,{"inputs":[{"name":"self"},{"name":"lengthfieldproto"}],"output":{"name":"bool"}}],[11,"new","","",6,{"inputs":[{"name":"usize"}],"output":{"name":"self"}}],[11,"bind_transport","","",6,null],[11,"bind_transport","","",6,null],[11,"fmt","","",7,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"clone","","",7,{"inputs":[{"name":"self"}],"output":{"name":"lengthfieldcodec"}}],[11,"new","","",7,{"inputs":[{"name":"usize"}],"output":{"name":"lengthfieldcodec"}}],[11,"decode","","",7,{"inputs":[{"name":"self"},{"name":"easybuf"}],"output":{"name":"result"}}],[11,"encode","","",7,{"inputs":[{"name":"self"},{"name":"vec"},{"name":"vec"}],"output":{"name":"result"}}],[0,"request_id_field","framecodecs","Converts pipelined protocol to multiplexed protocol by prepending a request id to each frame.",null,null],[3,"RequestIdFieldProto","framecodecs::request_id_field","A protocol that converts a pipelining codec into a multiplexing codec by prepending a `u64` request id field to every frame of the base codec.",null,null],[3,"RequestIdFieldCodec","","Protocol codec used by `RequestIdFieldProto`.",null,null],[11,"fmt","","",8,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"default","","",8,{"inputs":[],"output":{"name":"requestidfieldproto"}}],[11,"clone","","",8,{"inputs":[{"name":"self"}],"output":{"name":"requestidfieldproto"}}],[11,"new","","Creates a new `RequestIdFieldProto` from a base codec.",8,{"inputs":[{"name":"c"}],"output":{"name":"self"}}],[11,"bind_transport","","",8,null],[11,"bind_transport","","",8,null],[11,"fmt","","",9,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"clone","","",9,{"inputs":[{"name":"self"}],"output":{"name":"requestidfieldcodec"}}],[11,"default","","",9,{"inputs":[],"output":{"name":"requestidfieldcodec"}}],[11,"new","","",9,{"inputs":[{"name":"c"}],"output":{"name":"self"}}],[11,"decode","","",9,{"inputs":[{"name":"self"},{"name":"easybuf"}],"output":{"name":"result"}}],[11,"encode","","",9,null],[0,"remote_addr","framecodecs","Wrapper protocol for providing remote address of connection.",null,null],[3,"RemoteAddrProto","framecodecs::remote_addr","A wrapper protocol provides remote address of connection. This protocol implements only `ServerProto`.",null,null],[3,"RemoteAddrCodec","","Protocol codec used by `RemoteAddrProto`.",null,null],[11,"fmt","","",10,{"inputs":[{"name":"self"},{"name":"formatter"}],"output":{"name":"result"}}],[11,"clone","","",10,{"inputs":[{"name":"self"}],"output":{"name":"remoteaddrproto"}}],[11,"new","","",10,{"inputs":[{"name":"c"}],"output":{"name":"self"}}],[11,"bind_transport","","",10,null],[11,"bind_transport","","",10,null],[11,"new","","",11,{"inputs":[{"name":"c"},{"name":"socketaddr"}],"output":{"name":"self"}}],[11,"decode","","",11,{"inputs":[{"name":"self"},{"name":"easybuf"}],"output":{"name":"result"}}],[11,"encode","","",11,null],[11,"decode","","",11,{"inputs":[{"name":"self"},{"name":"easybuf"}],"output":{"name":"result"}}],[11,"encode","","",11,null]],"paths":[[3,"FixedLengthProto"],[3,"FixedLengthCodec"],[4,"LineDelimiter"],[8,"Delimiter"],[3,"DelimiterProto"],[3,"DelimiterCodec"],[3,"LengthFieldProto"],[3,"LengthFieldCodec"],[3,"RequestIdFieldProto"],[3,"RequestIdFieldCodec"],[3,"RemoteAddrProto"],[3,"RemoteAddrCodec"]]};
initSearch(searchIndex);
