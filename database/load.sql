USE research;

INSERT INTO lru_types (lru_type)
VALUES 
    ("Stack"),
    ("Vec"),
    ("Olken"),
    ("Scale");

INSERT INTO programs (program_name)
VALUES
    ("lu"),
    ("trmm_trace"),
    ("mvt"),
    ("trisolv"),
    ("syrk"),
    ("syr2d"),
    ("gemm"),
    ("3mm"),
    ("2mm"),
    ("cholesky"),
    ("gramschmidt_trace"),
    ("heat_3d"),
    ("convolution_2d"),
    ("symm"),
    ("stencil"),
    ("seidel_2d"),
    ("ludcmp"),
    ("nussinov"),
    ("jacobi_1d"),
    ("jacobi_2d"),
    ("gesummv"),
    ("gemver"),
    ("matmul");

INSERT INTO user_requests (email, user_name)
VALUES
    ("falsalih@u.rochester.edu", "falsalih");

INSERT INTO users (user_name, email, access_key)
VALUES
    ("falsalih", "falsalih@u.rochester.edu", "1");