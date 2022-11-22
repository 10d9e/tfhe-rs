//! Module containing functions related to LWE ciphertext encryption and decryption

use crate::core_crypto::algorithms::slice_algorithms::*;
use crate::core_crypto::commons::crypto::secret::generators::EncryptionRandomGenerator;
use crate::core_crypto::commons::math::random::ByteRandomGenerator;
use crate::core_crypto::commons::math::torus::UnsignedTorus;
use crate::core_crypto::commons::numeric::UnsignedInteger;
use crate::core_crypto::commons::traits::{Container, ContainerMut, *};
use crate::core_crypto::entities::*;
use crate::core_crypto::specification::dispersion::DispersionParameter;
use crate::core_crypto::specification::parameters::LweSize;

pub fn encrypt_lwe_ciphertext<Scalar, KeyCont, OutputCont, Gen>(
    lwe_secret_key: &LweSecretKeyBase<KeyCont>,
    output: &mut LweCiphertextBase<OutputCont>,
    encoded: Plaintext<Scalar>,
    noise_parameters: impl DispersionParameter,
    generator: &mut EncryptionRandomGenerator<Gen>,
) where
    Scalar: UnsignedTorus,
    KeyCont: Container<Element = Scalar>,
    OutputCont: ContainerMut<Element = Scalar>,
    Gen: ByteRandomGenerator,
{
    let (mut mask, body) = output.get_mut_mask_and_body();

    generator.fill_slice_with_random_mask(mask.as_mut());

    // generate an error from the normal distribution described by std_dev
    *body.0 = generator.random_noise(noise_parameters);

    // compute the multisum between the secret key and the mask
    *body.0 = (*body.0).wrapping_add(wrapping_dot_product(mask.as_ref(), lwe_secret_key.as_ref()));

    *body.0 = (*body.0).wrapping_add(encoded.0);
}

pub fn allocate_and_encrypt_new_lwe_ciphertext<Scalar, KeyCont, Gen>(
    lwe_secret_key: &LweSecretKeyBase<KeyCont>,
    encoded: Plaintext<Scalar>,
    noise_parameters: impl DispersionParameter,
    generator: &mut EncryptionRandomGenerator<Gen>,
) -> LweCiphertext<Scalar>
where
    Scalar: UnsignedTorus,
    KeyCont: Container<Element = Scalar>,
    Gen: ByteRandomGenerator,
{
    let mut new_ct = LweCiphertext::new(Scalar::ZERO, lwe_secret_key.lwe_dimension().to_lwe_size());

    encrypt_lwe_ciphertext(
        lwe_secret_key,
        &mut new_ct,
        encoded,
        noise_parameters,
        generator,
    );

    new_ct
}

pub fn trivially_encrypt_lwe_ciphertext<Scalar, OutputCont>(
    output: &mut LweCiphertextBase<OutputCont>,
    encoded: Plaintext<Scalar>,
) where
    Scalar: UnsignedTorus,
    OutputCont: ContainerMut<Element = Scalar>,
{
    output
        .as_mut()
        .iter_mut()
        .for_each(|elt| *elt = Scalar::ZERO);

    *output.get_mut_body().0 = encoded.0
}

pub fn allocate_and_trivially_encrypt_new_lwe_ciphertext<Scalar>(
    lwe_size: LweSize,
    encoded: Plaintext<Scalar>,
) -> LweCiphertext<Scalar>
where
    Scalar: UnsignedTorus,
{
    let mut new_ct = LweCiphertext::new(Scalar::ZERO, lwe_size);

    *new_ct.get_mut_body().0 = encoded.0;

    new_ct
}

pub fn decrypt_lwe_ciphertext<Scalar, KeyCont, InputCont>(
    lwe_secret_key: &LweSecretKeyBase<KeyCont>,
    lwe_ciphertext: &LweCiphertextBase<InputCont>,
) -> Plaintext<Scalar>
where
    Scalar: UnsignedInteger,
    KeyCont: Container<Element = Scalar>,
    InputCont: Container<Element = Scalar>,
{
    let (mask, body) = lwe_ciphertext.get_mask_and_body();

    Plaintext(
        body.0
            .wrapping_sub(wrapping_dot_product(mask.as_ref(), lwe_secret_key.as_ref())),
    )
}

pub fn encrypt_lwe_ciphertext_list<Scalar, KeyCont, OutputCont, InputCont, Gen>(
    lwe_secret_key: &LweSecretKeyBase<KeyCont>,
    output: &mut LweCiphertextListBase<OutputCont>,
    encoded: &PlaintextListBase<InputCont>,
    noise_parameters: impl DispersionParameter,
    generator: &mut EncryptionRandomGenerator<Gen>,
) where
    Scalar: UnsignedTorus,
    KeyCont: Container<Element = Scalar>,
    OutputCont: ContainerMut<Element = Scalar>,
    InputCont: Container<Element = Scalar>,
    Gen: ByteRandomGenerator,
{
    assert!(
        output.ciphertext_count().0 == encoded.plaintext_count().0,
        "Mismatch between number of output cipertexts and input plaintexts. \
        Got {:?} plaintexts, and {:?} ciphertext.",
        encoded.plaintext_count(),
        output.ciphertext_count()
    );

    for (encoded_plaintext_ref, mut ciphertext) in encoded.iter().zip(output.iter_mut()) {
        encrypt_lwe_ciphertext(
            lwe_secret_key,
            &mut ciphertext,
            encoded_plaintext_ref.into(),
            noise_parameters,
            generator,
        )
    }
}