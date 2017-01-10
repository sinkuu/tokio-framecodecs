(function() {var implementors = {};
implementors["framecodecs"] = ["impl&lt;B, C, T&gt; ServerProto&lt;T&gt; for <a class='struct' href='framecodecs/request_id_field/struct.RequestIdFieldProto.html' title='framecodecs::request_id_field::RequestIdFieldProto'>RequestIdFieldProto</a>&lt;B, C&gt; <span class='where fmt-newline'>where C: <a class='trait' href='https://docs.rs/tokio-core/0.1/tokio_core/io/frame/trait.Codec.html' title='tokio_core::io::frame::Codec'>Codec</a> + <a class='trait' href='https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html' title='core::clone::Clone'>Clone</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;B: ByteOrder + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;T: <a class='trait' href='https://docs.rs/tokio-core/0.1/tokio_core/io/trait.Io.html' title='tokio_core::io::Io'>Io</a> + 'static</span>","impl&lt;C, In, Out&gt; ServerProto&lt;<a class='struct' href='https://docs.rs/tokio-core/0.1/tokio_core/net/tcp/struct.TcpStream.html' title='tokio_core::net::tcp::TcpStream'>TcpStream</a>&gt; for <a class='struct' href='framecodecs/remote_addr/struct.RemoteAddrProto.html' title='framecodecs::remote_addr::RemoteAddrProto'>RemoteAddrProto</a>&lt;C, Multiplex&gt; <span class='where fmt-newline'>where C: <a class='trait' href='https://docs.rs/tokio-core/0.1/tokio_core/io/frame/trait.Codec.html' title='tokio_core::io::frame::Codec'>Codec</a>&lt;In=<a class='primitive' href='https://doc.rust-lang.org/nightly/std/primitive.tuple.html'>(</a>RequestId, In<a class='primitive' href='https://doc.rust-lang.org/nightly/std/primitive.tuple.html'>)</a>, Out=<a class='primitive' href='https://doc.rust-lang.org/nightly/std/primitive.tuple.html'>(</a>RequestId, Out<a class='primitive' href='https://doc.rust-lang.org/nightly/std/primitive.tuple.html'>)</a>&gt; + <a class='trait' href='https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html' title='core::clone::Clone'>Clone</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;In: 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;Out: 'static</span>",];

            if (window.register_implementors) {
                window.register_implementors(implementors);
            } else {
                window.pending_implementors = implementors;
            }
        
})()
