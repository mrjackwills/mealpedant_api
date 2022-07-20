import { createEmailTemplate } from '../../config/emailTemplate';
import { TestHelper } from '../testHelper';

const testHelper = new TestHelper();

import { describe, expect, it } from 'vitest';

describe('createEmailTemplate function', () => {

	it.concurrent('should return string with all params', async () => {
		expect.assertions(2);
		const result = createEmailTemplate(
			{
				title: testHelper.email_template_title,
				firstName: testHelper.email_template_title,
				lineOne: testHelper.email_template_lineOne,
				lineTwo: testHelper.email_template_lineTwo,
				buttonText: testHelper.email_template_buttonText,
				buttonLink: testHelper.email_template_buttonLink,
			}
		);
		expect(typeof result).toEqual('string');
		const strippedResult = result.replace(/\n/g, '').replace(/\s{2,}/g, ' ').trimStart();
		expect(strippedResult.startsWith(testHelper.email_template_htmlStarter)).toBeTruthy();
	});

	it.concurrent('should contain lineTwo', async () => {
		expect.assertions(2);
		const result = createEmailTemplate(
			{
				title: testHelper.email_template_title,
				firstName: testHelper.email_template_title,
				lineOne: testHelper.email_template_lineOne,
				lineTwo: testHelper.email_template_lineTwo,
				buttonText: testHelper.email_template_buttonText,
				buttonLink: testHelper.email_template_buttonLink,
			}
		);
		expect(typeof result).toEqual('string');
		expect(result.includes(testHelper.email_template_lineTwo)).toBeTruthy();
	});

	it.concurrent('should return string with just title, firstName, and lineOne', async () => {
		expect.assertions(1);
		const result = createEmailTemplate(
			{
				title: testHelper.email_template_title,
				firstName: testHelper.email_template_title,
				lineOne: testHelper.email_template_lineOne,
			}
		);
		expect(typeof result).toEqual('string');
	});

	it.concurrent('should return string with all params', async () => {
		expect.assertions(2);
		const result = createEmailTemplate(
			{
				title: testHelper.email_template_title,
				firstName: testHelper.email_template_title,
				lineOne: testHelper.email_template_lineOne,
				lineTwo: testHelper.email_template_lineTwo,
				buttonText: testHelper.email_template_buttonText,
				buttonLink: testHelper.email_template_buttonLink,
			});
		expect(typeof result).toEqual('string');
		expect(result.includes(testHelper.email_template_buttonHtml)).toBeTruthy();
	});

	it.concurrent('should throw error with no title', async () => {
		expect.assertions(2);
		try {
			createEmailTemplate(
				{
					// eslint-disable-next-line @typescript-eslint/ban-ts-comment
					// @ts-ignore
					title: '',
					firstName: testHelper.email_template_title,
					lineOne: testHelper.email_template_lineOne,
				});
		} catch (e) {
			if (e instanceof Error) {
				expect(e).toBeInstanceOf(TypeError);
				expect(e.message).toEqual(`createEmailTemplate: !title || !name || !lineOne`);
			}

		}
	});

	it.concurrent('should throw error with no first name', async () => {
		expect.assertions(2);
		try {
			createEmailTemplate(
				{
					title: testHelper.email_template_title,
					// eslint-disable-next-line @typescript-eslint/ban-ts-comment
					// @ts-ignore
					firstName: '',
					lineOne: testHelper.email_template_lineOne,
				});
		} catch (e) {
			if (e instanceof Error) {
				expect(e).toBeInstanceOf(TypeError);
				expect(e.message).toEqual(`createEmailTemplate: !title || !name || !lineOne`);
			}
		}
	});

	it.concurrent('should throw error with no line one', async () => {
		expect.assertions(2);
		try {
			createEmailTemplate(
				{
					title: testHelper.email_template_title,
					firstName: testHelper.email_template_lineOne,
					// eslint-disable-next-line @typescript-eslint/ban-ts-comment
					// @ts-ignore
					lineOne: '',
				});
		} catch (e) {
			if (e instanceof Error) {
				expect(e).toBeInstanceOf(TypeError);
				expect(e.message).toEqual(`createEmailTemplate: !title || !name || !lineOne`);
			}

		}
	});

	it.concurrent('should throw error with buttonText but no buttonLink', async () => {
		expect.assertions(2);
		try {
			createEmailTemplate(
				{
					title: testHelper.email_template_title,
					firstName: testHelper.email_template_lineOne,
					lineOne: testHelper.email_template_lineOne,
					buttonText: testHelper.email_template_buttonText,
					buttonLink: ''
				});
		} catch (e) {
			if (e instanceof Error) {
				expect(e).toBeInstanceOf(TypeError);
				expect(e.message).toEqual(`createEmailTemplate: buttonText && !buttonLink || !buttonText && buttonLink`);
			}

		}
	});

	it.concurrent('should throw error with buttonLink but no buttonTex', async () => {
		expect.assertions(2);
		try {
			createEmailTemplate(
				{
					title: testHelper.email_template_title,
					firstName: testHelper.email_template_lineOne,
					lineOne: testHelper.email_template_lineOne,
					// eslint-disable-next-line @typescript-eslint/ban-ts-comment
					// @ts-ignore
					buttonText: '',
					buttonLink: testHelper.email_template_buttonLink
				});
		} catch (e) {
			if (e instanceof Error) {
				expect(e).toBeInstanceOf(TypeError);
				expect(e.message).toEqual(`createEmailTemplate: buttonText && !buttonLink || !buttonText && buttonLink`);
			}

		}
	});

});